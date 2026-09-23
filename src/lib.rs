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
        .route("/register", post(user::register_user))
        .route("/login", post(user::login_user))
        .layer(rate_limit_layer);

    build_app(state, auth_routes)
}

/// Build the router without rate limiting (for integration tests).
pub fn build_router_without_rate_limit(state: AppState) -> Router {
    let auth_routes = Router::new()
        .route("/register", post(user::register_user))
        .route("/login", post(user::login_user));

    build_app(state, auth_routes)
}

fn build_app(state: AppState, auth_routes: Router<AppState>) -> Router {
    let sensor_write_routes = Router::new()
        .route("/", post(reading::create_reading))
        .route_layer(axum_mw::from_fn_with_state(
            state.clone(),
            middleware::auth::require_admin,
        ));

    let sensor_routes = Router::new()
        .route("/", get(reading::list_readings))
        .route("/:id", get(reading::get_reading))
        .merge(sensor_write_routes);

    Router::new()
        .nest("/auth", auth_routes)
        .nest("/sensors", sensor_routes)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

// End of File
