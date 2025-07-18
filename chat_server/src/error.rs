use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorOutput {
    pub error: String,
}

#[derive(Error, Debug)]
pub enum AppError {
    #[error("internal server error")]
    SqlxError(#[from] sqlx::Error),

    #[error("create chat error: {0}")]
    CreateChatError(String),

    #[error("update chat error: {0}")]
    UpdateChatError(String),

    #[error("delete chat error: {0}")]
    DeleteChatError(String),

    #[error("{0}")]
    ChatFileError(String),

    #[error("create message error: {0}")]
    CreateMessageError(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("password hash error")]
    PasswordHashError(#[from] argon2::password_hash::Error),

    #[error("email already exists: {0}")]
    EmailAlreadyExists(String),

    #[error("jwt error: {0}")]
    JwtError(#[from] jwt_simple::Error),

    #[error("invalid credentials")]
    InvalidCredentials,

    #[error("http header parse error")]
    HttpHeaderError(#[from] axum::http::header::InvalidHeaderValue),
}

impl ErrorOutput {
    pub fn new(error: impl Into<String>) -> Self {
        Self {
            error: error.into(),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response<axum::body::Body> {
        let status = match &self {
            Self::SqlxError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::CreateChatError(_) => StatusCode::BAD_REQUEST,
            Self::UpdateChatError(_) => StatusCode::BAD_REQUEST,
            Self::DeleteChatError(_) => StatusCode::BAD_REQUEST,
            Self::ChatFileError(_) => StatusCode::BAD_REQUEST,
            Self::CreateMessageError(_) => StatusCode::BAD_REQUEST,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::IoError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::PasswordHashError(_) => StatusCode::UNPROCESSABLE_ENTITY,
            Self::EmailAlreadyExists(_) => StatusCode::CONFLICT,
            Self::JwtError(_) => StatusCode::FORBIDDEN,
            Self::HttpHeaderError(_) => StatusCode::UNPROCESSABLE_ENTITY,
            Self::InvalidCredentials => StatusCode::CONFLICT,
        };
        let body = Json(ErrorOutput::new(self.to_string()));
        (status, body).into_response()
    }
}
