//! src/db.rs — Database initialization and migration runner.
//!

use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use std::str::FromStr;

/// Creates the SQLite connection pool and runs migrations.
pub async fn init_db(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    let is_memory = database_url.contains(":memory:");
    let options = SqliteConnectOptions::from_str(database_url)?.create_if_missing(true);

    // For in-memory databases (tests), use a single connection so all
    // queries share the same database. For file-based DBs, use a pool.
    let max_conn = if is_memory { 1 } else { 5 };

    let pool = SqlitePoolOptions::new()
        .max_connections(max_conn)
        .connect_with(options)
        .await?;

    // Run migrations — split into individual statements because
    // sqlx::query().execute() only supports one statement at a time.
    let migration = include_str!("../migrations/001_init.sql");
    for statement in migration.split(';') {
        let trimmed = statement.trim();
        if !trimmed.is_empty() {
            sqlx::query(trimmed).execute(&pool).await?;
        }
    }

    tracing::info!("Database initialized successfully");
    Ok(pool)
}

// End of File
