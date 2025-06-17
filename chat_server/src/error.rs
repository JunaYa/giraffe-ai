use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Internal server error")]
    SqlxError(#[from] sqlx::Error),

    #[error("Password hash error")]
    PasswordHashError(#[from] argon2::password_hash::Error),
}
