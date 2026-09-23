//! src/error/mod.rs — Centralized error handling with safe client responses.
//!

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    // SECURITY: Both "user not found" and "wrong password" map to the same
    // client-facing message. An attacker cannot distinguish between them,
    // blocking user-enumeration attacks (OWASP A07:2021).
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
        // SECURITY: Log the real error server-side for debugging,
        // but return a generic message to the client.
        let (status, message) = match &self {
            AppError::InvalidCredentials => {
                // Same message for wrong user AND wrong password.
                (StatusCode::UNAUTHORIZED, "Invalid email or password")
            }
            AppError::UserAlreadyExists => (StatusCode::CONFLICT, "User already exists"),
            AppError::Validation(msg) => {
                // Validation errors are safe to return — they describe
                // input format issues, not internal state.
                return (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    Json(json!({ "error": msg })),
                )
                    .into_response();
            }
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "Authentication required"),
            AppError::Forbidden => (StatusCode::FORBIDDEN, "Insufficient permissions"),
            AppError::NotFound => (StatusCode::NOT_FOUND, "Resource not found"),
            AppError::Internal(detail) => {
                // SECURITY: Log the real error but never send it to the client.
                tracing::error!(error = %detail, "Internal server error");
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
            AppError::Database(e) => {
                tracing::error!(error = %e, "Database error");
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
        };

        (status, Json(json!({ "error": message }))).into_response()
    }
}

// End of File
