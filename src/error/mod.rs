//! src/error/mod.rs — Centralized error handling with safe client responses.
//!

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

const MSG_INVALID_CREDENTIALS: &str = "Invalid email or password";
const MSG_USER_ALREADY_EXISTS: &str = "User already exists";
const MSG_AUTH_REQUIRED: &str = "Authentication required";
const MSG_INSUFFICIENT_PERMISSIONS: &str = "Insufficient permissions";
const MSG_NOT_FOUND: &str = "Resource not found";
const MSG_INTERNAL_ERROR: &str = "Internal server error";

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("User already exists")]
    UserAlreadyExists,

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Forbidden")]
    Forbidden,

    #[error("Not found")]
    NotFound,

    #[error("Internal error")]
    Internal(String),

    #[error("Database error")]
    Database(#[from] sqlx::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::InvalidCredentials => (StatusCode::UNAUTHORIZED, MSG_INVALID_CREDENTIALS),
            AppError::UserAlreadyExists => (StatusCode::CONFLICT, MSG_USER_ALREADY_EXISTS),
            AppError::Validation(msg) => {
                return (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    Json(json!({ "error": msg })),
                )
                    .into_response();
            }
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, MSG_AUTH_REQUIRED),
            AppError::Forbidden => (StatusCode::FORBIDDEN, MSG_INSUFFICIENT_PERMISSIONS),
            AppError::NotFound => (StatusCode::NOT_FOUND, MSG_NOT_FOUND),
            AppError::Internal(detail) => {
                tracing::error!(error = %detail, MSG_INTERNAL_ERROR);
                (StatusCode::INTERNAL_SERVER_ERROR, MSG_INTERNAL_ERROR)
            }
            AppError::Database(e) => {
                tracing::error!(error = %e, "Database error");
                (StatusCode::INTERNAL_SERVER_ERROR, MSG_INTERNAL_ERROR)
            }
        };

        (status, Json(json!({ "error": message }))).into_response()
    }
}

// End of File
