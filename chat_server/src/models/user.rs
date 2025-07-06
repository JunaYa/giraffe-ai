use std::mem;

use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use serde::{Deserialize, Serialize};

use super::ChatUser;
use crate::{AppError, AppState, User};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUser {
    pub fullname: String,
    pub email: String,
    pub workspace: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigninUser {
    pub email: String,
    pub password: String,
}

#[allow(dead_code)]
impl AppState {
    pub async fn find_user_by_id(&self, id: i64) -> Result<Option<User>, AppError> {
        let user = sqlx::query_as(
            r#"
            SELECT id, ws_id, fullname, email, created_at
            FROM users
            WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn find_user_by_email(&self, email: &str) -> Result<Option<User>, AppError> {
        let user = sqlx::query_as(
            r#"
            SELECT id, ws_id, fullname, email, created_at
            FROM users
            WHERE email = $1"#,
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn create_user(&self, input: &CreateUser) -> Result<User, AppError> {
        // check if user already exists
        let user = self.find_user_by_email(&input.email).await?;
        if user.is_some() {
            return Err(AppError::EmailAlreadyExists(input.email.clone()));
        }

        // check if workspace already exists, if not create it
        let workspace = match self.find_workspace_by_name(&input.workspace).await? {
            Some(ws) => ws,
            None => self.create_workspace(&input.workspace, 0).await?,
        };

        let password_hash = hash_password(&input.password)?;

        let user: User = sqlx::query_as(
            r#"
            INSERT INTO users (ws_id, email, fullname, password_hash)
            VALUES ($1, $2, $3, $4)
            RETURNING id, ws_id, email, fullname, created_at
            "#,
        )
        .bind(workspace.id)
        .bind(&input.email)
        .bind(&input.fullname)
        .bind(password_hash)
        .fetch_one(&self.pool)
        .await?;

        if workspace.owner_id == 0 {
            workspace.update_owner(user.id as _, &self.pool).await?;
        }

        Ok(user)
    }

    pub async fn verify_user(&self, input: &SigninUser) -> Result<Option<User>, AppError> {
        let user: Option<User> = sqlx::query_as(
            r#"
            SELECT id, ws_id, fullname, email, password_hash, created_at
            FROM users
            WHERE email = $1
            "#,
        )
        .bind(&input.email)
        .fetch_optional(&self.pool)
        .await?;

        match user {
            Some(mut user) => {
                let password_hash = mem::take(&mut user.password_hash);
                let is_valid =
                    verify_password(&input.password, &password_hash.unwrap_or_default())?;
                if is_valid { Ok(Some(user)) } else { Ok(None) }
            }
            None => Ok(None),
        }
    }

    pub async fn fetch_by_ids(&self, ids: &[i64]) -> Result<Vec<ChatUser>, AppError> {
        let users = sqlx::query_as(
            r#"
            SELECT id, fullname, email
            FROM users
            WHERE id = ANY($1)
            "#,
        )
        .bind(ids)
        .fetch_all(&self.pool)
        .await?;

        Ok(users)
    }

    pub async fn fetch_chat_users(&self, ws_id: i64) -> Result<Vec<ChatUser>, AppError> {
        let users = sqlx::query_as(
            r#"
            SELECT id, fullname, email
            FROM users
            WHERE ws_id = $1
            "#,
        )
        .bind(ws_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(users)
    }
}

fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    // Argon2 with default params (Argon2id v19)
    let argon2 = Argon2::default();

    // Hash password to PHC string ($argon2id$v=19$m=19456,t=2,p=1$...)
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)?
        .to_string();

    Ok(password_hash)
}

fn verify_password(password: &str, password_hash: &str) -> Result<bool, AppError> {
    let argon2 = argon2::Argon2::default();
    let password_hash = PasswordHash::new(password_hash)?;
    let is_valid = argon2
        .verify_password(password.as_bytes(), &password_hash)
        .is_ok();
    Ok(is_valid)
}

#[cfg(test)]
impl User {
    pub fn new(id: i64, fullname: &str, email: &str) -> Self {
        Self {
            id,
            fullname: fullname.to_string(),
            email: email.to_string(),
            ws_id: 0,
            password_hash: None,
            created_at: chrono::Utc::now(),
        }
    }
}

#[cfg(test)]
impl CreateUser {
    pub fn new(workspace: &str, fullname: &str, email: &str, password: &str) -> Self {
        Self {
            fullname: fullname.to_string(),
            email: email.to_string(),
            password: password.to_string(),
            workspace: workspace.to_string(),
        }
    }
}

#[cfg(test)]
impl SigninUser {
    pub fn new(email: &str, password: &str) -> Self {
        Self {
            email: email.to_string(),
            password: password.to_string(),
        }
    }
}
#[cfg(test)]
mod tests {

    use super::*;
    use anyhow::Result;

    #[test]
    fn hash_password_and_verify_should_work() -> Result<()> {
        let password = "password";
        let password_hash = hash_password(password)?;
        assert_eq!(password_hash.len(), 97);
        let is_valid = verify_password(password, &password_hash)?;
        assert!(is_valid);
        Ok(())
    }

    #[tokio::test]
    async fn create_and_verify_user_should_work() -> Result<()> {
        // _tdb 没有被使用，但是要显式返回，需要 drop 时 drop 掉数据库
        let (_tdb, state) = AppState::new_for_test().await?;

        let input = CreateUser::new("acme", "Arjun001", "arjun001@acme.org", "hunter42");
        let user = state.create_user(&input).await?;
        assert_eq!(user.email, input.email);
        assert_eq!(user.fullname, input.fullname);
        assert!(user.id > 0);

        let user = state.find_user_by_email(&input.email).await?;

        assert!(user.is_some());
        let user = user.unwrap();
        assert_eq!(user.email, input.email);
        assert_eq!(user.fullname, input.fullname);
        let input = SigninUser::new(&input.email, &input.password);
        let user = state.verify_user(&input).await?;
        assert!(user.is_some());

        Ok(())
    }

    #[tokio::test]
    async fn create_duplicate_user_should_fail() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;

        let input = CreateUser::new("acme", "Arjun001", "arjun001@acme.org", "hunter42");
        state.create_user(&input).await?;
        let ret = state.create_user(&input).await;
        match ret {
            Err(AppError::EmailAlreadyExists(email)) => {
                assert_eq!(email, input.email);
            }
            _ => panic!("Expecting EmailAlreadyExists error"),
        }
        Ok(())
    }

    #[tokio::test]
    async fn find_user_by_id_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let user = state.find_user_by_id(1).await?;
        assert!(user.is_some());
        let user = user.unwrap();
        assert_eq!(user.id, 1);
        Ok(())
    }
}
