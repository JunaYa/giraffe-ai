mod config;
mod error;
mod handlers;
mod middleware;
mod models;
mod utils;

use anyhow::Context;
use handlers::*;
use sqlx::PgPool;
use std::{fmt, ops::Deref, sync::Arc};

pub use error::{AppError, ErrorOutput};
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

const REQUEST_ID_HEADER: &str = "x-request-id";

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
        .route("/chat", get(list_chat_handler).post(create_chat_handler))
        .route(
            "/chat/{id}",
            patch(update_chat_handler)
                .delete(delete_chat_handler)
                .post(send_message_handler),
        )
        .route("/chat/{id}/messages", get(list_message_handler))
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
impl AppState {
    pub async fn new_for_test(
        config: AppConfig,
    ) -> Result<(sqlx_db_tester::TestPg, Self), AppError> {
        use sqlx_db_tester::TestPg;
        let dk = DecodingKey::load(&config.auth.pk).context("failed to load dk")?;
        let ek = EncodingKey::load(&config.auth.sk).context("failed to load ek")?;
        // Parse the database URL to get the base URL without the database name
        // Format is typically: postgresql://username:password@hostname:port/database
        // We want everything before the last slash
        let db_url = &config.server.db_url;
        let base_url = match db_url.rfind('/') {
            Some(pos) => &db_url[0..pos],
            None => db_url, // Fallback if no slash found
        };

        let tdb = TestPg::new(base_url.to_string(), std::path::Path::new("../migrations"));
        let pool = tdb.get_pool().await;
        let state = Self {
            inner: Arc::new(AppStateInner {
                config,
                pool,
                ek,
                dk,
            }),
        };
        Ok((tdb, state))
    }
}
