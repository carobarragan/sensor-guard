//! src/middleware/auth.rs — JWT authentication extractor and RBAC enforcement.
//!

use axum::{
    extract::{FromRequestParts, Request},
    http::request::Parts,
    middleware::Next,
    response::Response,
};

use crate::enums::Role;
use crate::error::AppError;
use crate::models::Claims;
use crate::AppState;

const HEADER_AUTHORIZATION: &str = "Authorization";
const BEARER_PREFIX: &str = "Bearer ";

/// Extractor that validates the JWT and provides the authenticated user's claims.
#[derive(Debug, Clone)]
pub struct AuthUser(pub Claims);

#[axum::async_trait]
impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get(HEADER_AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or(AppError::Unauthorized)?;

        let token = auth_header
            .strip_prefix(BEARER_PREFIX)
            .ok_or(AppError::Unauthorized)?;

        let claims = crate::auth::decode_token(token, &state.jwt_secret)
            .map_err(|_| AppError::Unauthorized)?;

        Ok(AuthUser(claims))
    }
}

/// Middleware that requires admin role.
pub async fn require_admin(
    AuthUser(claims): AuthUser,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    if claims.role != Role::Admin {
        tracing::warn!(
            user_id = %claims.sub,
            role = %claims.role,
            path = %request.uri().path(),
            "Access denied: admin role required"
        );
        return Err(AppError::Forbidden);
    }
    Ok(next.run(request).await)
}

// End of File
