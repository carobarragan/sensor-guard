//! src/reading/reading_repository.rs — Data access for the sensor_readings table.
//!

#[cfg(test)]
use mockall::automock;
use sqlx::SqlitePool;

use crate::error::AppError;
use crate::models::SensorReading;

#[cfg_attr(test, automock)]
#[async_trait::async_trait]
pub trait ReadingRepository: Send + Sync {
    async fn list_all(&self, limit: i64) -> Result<Vec<SensorReading>, AppError>;
    async fn find_by_id(&self, id: &str) -> Result<Option<SensorReading>, AppError>;
    async fn create(&self, reading: &SensorReading) -> Result<(), AppError>;
}

pub struct SqliteReadingRepository {
    pool: SqlitePool,
}

impl SqliteReadingRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl ReadingRepository for SqliteReadingRepository {
    async fn list_all(&self, limit: i64) -> Result<Vec<SensorReading>, AppError> {
        let readings = sqlx::query_as::<_, SensorReading>(
            "SELECT id, sensor_name, value, unit, recorded_at, created_by \
             FROM sensor_readings ORDER BY recorded_at DESC LIMIT ?",
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
        Ok(readings)
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<SensorReading>, AppError> {
        let reading = sqlx::query_as::<_, SensorReading>(
            "SELECT id, sensor_name, value, unit, recorded_at, created_by \
             FROM sensor_readings WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
        Ok(reading)
    }

    async fn create(&self, reading: &SensorReading) -> Result<(), AppError> {
        sqlx::query(
            "INSERT INTO sensor_readings (id, sensor_name, value, unit, recorded_at, created_by) \
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(&reading.id)
        .bind(&reading.sensor_name)
        .bind(reading.value)
        .bind(&reading.unit)
        .bind(&reading.recorded_at)
        .bind(&reading.created_by)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
        Ok(())
    }
}

// End of File
