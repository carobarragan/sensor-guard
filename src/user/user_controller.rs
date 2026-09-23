//! src/user/user_controller.rs — Thin handlers for user registration and login.
//!

use axum::{extract::State, Json};

use crate::error::AppError;
use crate::models::{AuthResponse, LoginRequest, RegisterRequest, UserInfo};
use crate::AppState;

/// POST /auth/register
pub async fn register_user(
    State(state): State<AppState>,
    Json(body): Json<RegisterRequest>,
) -> Result<Json<UserInfo>, AppError> {
    let info = state.user_service.register_user(body).await?;
    Ok(Json(info))
}

/// POST /auth/login
pub async fn login_user(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let auth = state
        .user_service
        .login_user(body, &state.jwt_secret)
        .await?;
    Ok(Json(auth))
}

// End of File
