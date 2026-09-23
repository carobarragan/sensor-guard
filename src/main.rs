//! src/main.rs — Application entry point: loads config, initializes DB, starts server.
//!

use std::sync::Arc;

use sensor_guard::reading::reading_repository::SqliteReadingRepository;
use sensor_guard::reading::reading_service::ReadingServiceImpl;
use sensor_guard::user::user_repository::SqliteUserRepository;
use sensor_guard::user::user_service::UserServiceImpl;
use sensor_guard::{build_router, AppState};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = dotenvy::dotenv();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "sensor_guard=info,tower_http=info".into()),
        )
        .init();

    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://sensor-guard.db".to_string());
    let jwt_secret =
        std::env::var("JWT_SECRET").expect("JWT_SECRET must be set — see .env.example");
    let bind_addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string());

    let pool = sensor_guard::db::init_db(&database_url).await?;

    let user_repo = Arc::new(SqliteUserRepository::new(pool.clone()));
    let reading_repo = Arc::new(SqliteReadingRepository::new(pool));

    let state = AppState {
        jwt_secret,
        user_service: Arc::new(UserServiceImpl::new(user_repo)),
        reading_service: Arc::new(ReadingServiceImpl::new(reading_repo)),
    };

    let app = build_router(state);

    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    tracing::info!("Listening on {}", bind_addr);
    axum::serve(listener, app).await?;

    Ok(())
}

// End of File
