//! src/lib.rs — sensor-guard library crate, exposes types for integration tests.
//!

pub mod auth;
pub mod db;
pub mod enums;
pub mod error;
pub mod middleware;
pub mod models;
pub mod reading;
pub mod user;

use axum::{
    middleware as axum_mw,
    routing::{get, post},
    Router,
};
use reading::DynReadingService;
use tower_governor::GovernorLayer;
use tower_http::trace::TraceLayer;
use user::DynUserService;

// ── Route path constants ─────────────────────────────────────────

pub const PATH_AUTH: &str = "/auth";
pub const PATH_REGISTER: &str = "/register";
pub const PATH_LOGIN: &str = "/login";
pub const PATH_SENSORS: &str = "/sensors";
pub const PATH_SENSOR_BY_ID: &str = "/:id";
pub const PATH_ROOT: &str = "/";

/// Shared application state.
#[derive(Clone)]
pub struct AppState {
    pub jwt_secret: String,
    pub user_service: DynUserService,
    pub reading_service: DynReadingService,
}

/// Build the full router with rate limiting (for production use).
pub fn build_router(state: AppState) -> Router {
    let rate_limit_config = middleware::rate_limit::auth_rate_limiter();
    let rate_limit_layer = GovernorLayer {
        config: std::sync::Arc::new(rate_limit_config),
    };

    let auth_routes = Router::new()
        .route(PATH_REGISTER, post(user::register_user))
        .route(PATH_LOGIN, post(user::login_user))
        .layer(rate_limit_layer);

    build_app(state, auth_routes)
}

/// Build the router without rate limiting (for integration tests).
pub fn build_router_without_rate_limit(state: AppState) -> Router {
    let auth_routes = Router::new()
        .route(PATH_REGISTER, post(user::register_user))
        .route(PATH_LOGIN, post(user::login_user));

    build_app(state, auth_routes)
}

fn build_app(state: AppState, auth_routes: Router<AppState>) -> Router {
    let sensor_write_routes = Router::new()
        .route(PATH_ROOT, post(reading::create_reading))
        .route_layer(axum_mw::from_fn_with_state(
            state.clone(),
            middleware::auth::require_admin,
        ));

    let sensor_routes = Router::new()
        .route(PATH_ROOT, get(reading::list_readings))
        .route(PATH_SENSOR_BY_ID, get(reading::get_reading))
        .merge(sensor_write_routes);

    Router::new()
        .nest(PATH_AUTH, auth_routes)
        .nest(PATH_SENSORS, sensor_routes)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

// End of File
