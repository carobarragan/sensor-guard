//! src/reading/reading_controller.rs — Thin handlers for sensor reading CRUD.
//!

use axum::{
    extract::{Path, State},
    Json,
};

use crate::error::AppError;
use crate::middleware::auth::AuthUser;
use crate::models::{CreateReadingRequest, SensorReading};
use crate::AppState;

/// GET /sensors
pub async fn list_readings(
    _user: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<SensorReading>>, AppError> {
    let readings = state.reading_service.list().await?;
    Ok(Json(readings))
}

/// GET /sensors/:id
pub async fn get_reading(
    _user: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<SensorReading>, AppError> {
    let reading = state.reading_service.get_by_id(&id).await?;
    Ok(Json(reading))
}

/// POST /sensors (admin only, enforced by middleware)
pub async fn create_reading(
    AuthUser(claims): AuthUser,
    State(state): State<AppState>,
    Json(body): Json<CreateReadingRequest>,
) -> Result<(axum::http::StatusCode, Json<SensorReading>), AppError> {
    let reading = state.reading_service.create(&claims, body).await?;
    Ok((axum::http::StatusCode::CREATED, Json(reading)))
}

// End of File
