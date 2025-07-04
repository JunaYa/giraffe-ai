mod config;
mod error;
mod handlers;
mod middleware;
mod models;
mod utils;

use anyhow::Context;
use handlers::*;
use sqlx::PgPool;
use tokio::fs;

use std::{fmt, ops::Deref, sync::Arc};

pub use error::{AppError, ErrorOutput};
pub use models::ChatFile;
pub use models::User;

use axum::{
    Router,
    middleware::from_fn_with_state,
    routing::{get, patch, post},
};

pub use config::AppConfig;

use crate::{
    middleware::{set_layer, verify_token},
    utils::{DecodingKey, EncodingKey},
};

#[derive(Debug, Clone)]
pub(crate) struct AppState {
    inner: Arc<AppStateInner>,
}

#[allow(unused)]
pub(crate) struct AppStateInner {
    pub(crate) config: AppConfig,
    pub(crate) pool: PgPool,
    pub(crate) ek: EncodingKey,
    pub(crate) dk: DecodingKey,
}

pub async fn get_router(config: AppConfig) -> Result<Router, AppError> {
    let state = AppState::try_new(config).await?;

    let api = Router::new()
        .route("/users", get(list_chat_users_handler))
        .route("/chat", get(list_chat_handler).post(create_chat_handler))
        .route(
            "/chat/{id}",
            patch(update_chat_handler)
                .delete(delete_chat_handler)
                .post(send_message_handler),
        )
        .route("/chat/{id}/messages", get(list_message_handler))
        .route("/upload", post(upload_handler))
        .route("/files/{ws_id}/{*path}", get(file_handler))
        .layer(from_fn_with_state(state.clone(), verify_token))
        .route("/signin", post(signin_handler))
        .route("/signup", post(signup_handler));

    let app = Router::new()
        .route("/", get(index_handler))
        .nest("/api", api)
        .with_state(state);

    Ok(set_layer(app))
}

// 当我调用 state.config => state.inner.config
impl Deref for AppState {
    type Target = AppStateInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl AppState {
    pub async fn try_new(config: AppConfig) -> Result<Self, AppError> {
        fs::create_dir_all(&config.server.base_dir)
            .await
            .context("failed to create base dir")?;
        let ek = EncodingKey::load(&config.auth.sk).context("failed to load ek")?;
        let dk = DecodingKey::load(&config.auth.pk).context("failed to load dk")?;
        let pool = PgPool::connect(&config.server.db_url)
            .await
            .context("failed to connect to db")?;

        Ok(Self {
            inner: Arc::new(AppStateInner {
                config,
                pool,
                ek,
                dk,
            }),
        })
    }
}

impl fmt::Debug for AppStateInner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AppStateInner")
            .field("config", &self.config)
            .finish()
    }
}

#[cfg(test)]
mod test_utils {
    use super::*;
    use sqlx::{Executor, PgPool};
    use sqlx_db_tester::TestPg;

    impl AppState {
        pub async fn new_for_test(config: AppConfig) -> Result<(TestPg, Self), AppError> {
            let dk = DecodingKey::load(&config.auth.pk).context("failed to load dk")?;
            let ek = EncodingKey::load(&config.auth.sk).context("failed to load ek")?;
            let db_url = &config.server.db_url;
            let post = db_url.rfind('/').expect("invalid db_url");
            let server_url = &config.server.db_url[0..post];
            let (_tdb, pool) = get_test_pool(Some(server_url)).await;
            let state = Self {
                inner: Arc::new(AppStateInner {
                    config,
                    pool,
                    ek,
                    dk,
                }),
            };
            Ok((_tdb, state))
        }
    }

    pub async fn get_test_pool(url: Option<&str>) -> (TestPg, PgPool) {
        let url = match url {
            Some(url) => url.to_string(),
            None => "postgres://postgres:password@localhost:5432".to_string(),
        };
        let tdb = TestPg::new(url, std::path::Path::new("../migrations"));
        let pool = tdb.get_pool().await;

        // run prepared sql to insert test dat
        let sql = include_str!("../fixtures/test.sql").split(';');
        let mut ts = pool.begin().await.expect("begin transaction failed");
        for s in sql {
            if s.trim().is_empty() {
                continue;
            }
            ts.execute(s).await.expect("execute sql failed");
        }
        ts.commit().await.expect("commit transaction failed");

        (tdb, pool)
    }
}
