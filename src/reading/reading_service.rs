//! src/reading/reading_service.rs — Business logic for sensor readings.
//!

#[cfg(test)]
use mockall::automock;
use validator::Validate;

use crate::error::AppError;
use crate::models::{Claims, CreateReadingRequest, SensorReading};

use super::DynReadingRepo;

#[cfg_attr(test, automock)]
#[async_trait::async_trait]
pub trait ReadingService: Send + Sync {
    async fn list(&self) -> Result<Vec<SensorReading>, AppError>;
    async fn get_by_id(&self, id: &str) -> Result<SensorReading, AppError>;
    async fn create(
        &self,
        claims: &Claims,
        req: CreateReadingRequest,
    ) -> Result<SensorReading, AppError>;
}

pub struct ReadingServiceImpl {
    reading_repo: DynReadingRepo,
}

impl ReadingServiceImpl {
    pub fn new(reading_repo: DynReadingRepo) -> Self {
        Self { reading_repo }
    }
}

#[async_trait::async_trait]
impl ReadingService for ReadingServiceImpl {
    async fn list(&self) -> Result<Vec<SensorReading>, AppError> {
        tracing::info!("list_readings: start");
        let readings = self.reading_repo.list_all(100).await?;
        tracing::info!(count = readings.len(), "list_readings: completed");
        Ok(readings)
    }

    async fn get_by_id(&self, id: &str) -> Result<SensorReading, AppError> {
        tracing::info!(id = %id, "get_reading: start");
        match self.reading_repo.find_by_id(id).await? {
            Some(reading) => {
                tracing::info!(id = %id, "get_reading: completed");
                Ok(reading)
            }
            None => {
                tracing::warn!(id = %id, "get_reading: not found");
                Err(AppError::NotFound)
            }
        }
    }

    async fn create(
        &self,
        claims: &Claims,
        req: CreateReadingRequest,
    ) -> Result<SensorReading, AppError> {
        tracing::info!(user = %claims.sub, sensor = %req.sensor_name, "create_reading: start");

        req.validate()
            .map_err(|e| AppError::Validation(e.to_string()))?;

        let reading = SensorReading {
            id: uuid::Uuid::new_v4().to_string(),
            sensor_name: req.sensor_name,
            value: req.value,
            unit: req.unit,
            recorded_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            created_by: claims.sub.clone(),
        };

        self.reading_repo.create(&reading).await?;

        tracing::info!(id = %reading.id, "create_reading: completed");
        Ok(reading)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::Role;
    use crate::reading::reading_repository::MockReadingRepository;
    use std::sync::Arc;

    const STUB_USER_ID: &str = "user-1";

    fn make_reading_service(mock: MockReadingRepository) -> ReadingServiceImpl {
        ReadingServiceImpl::new(Arc::new(mock))
    }

    fn make_claims() -> Claims {
        Claims {
            sub: STUB_USER_ID.to_string(),
            email: "admin@test.com".to_string(),
            role: Role::Admin,
            exp: 9999999999,
            iat: 0,
        }
    }

    fn make_stub_reading() -> SensorReading {
        SensorReading {
            id: "r-1".to_string(),
            sensor_name: "temp-01".to_string(),
            value: 25.5,
            unit: "°C".to_string(),
            recorded_at: "2024-01-01 00:00:00".to_string(),
            created_by: STUB_USER_ID.to_string(),
        }
    }

    #[tokio::test]
    async fn when_list_should_return_readings() {
        let mut mock = MockReadingRepository::new();
        mock.expect_list_all()
            .returning(|_| Ok(vec![make_stub_reading()]));

        let svc = make_reading_service(mock);
        let result = svc.list().await.unwrap();
        assert_eq!(result.len(), 1);
    }

    #[tokio::test]
    async fn when_get_existing_should_return_reading() {
        let mut mock = MockReadingRepository::new();
        mock.expect_find_by_id()
            .returning(|_| Ok(Some(make_stub_reading())));

        let svc = make_reading_service(mock);
        let result = svc.get_by_id("r-1").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn when_get_nonexistent_should_return_not_found() {
        let mut mock = MockReadingRepository::new();
        mock.expect_find_by_id().returning(|_| Ok(None));

        let svc = make_reading_service(mock);
        let result = svc.get_by_id("nope").await;
        assert!(matches!(result, Err(AppError::NotFound)));
    }

    #[tokio::test]
    async fn when_create_should_persist_and_return() {
        let mut mock = MockReadingRepository::new();
        mock.expect_create().returning(|_| Ok(()));

        let svc = make_reading_service(mock);
        let req = CreateReadingRequest {
            sensor_name: "temp-01".to_string(),
            value: 25.5,
            unit: "°C".to_string(),
        };
        let result = svc.create(&make_claims(), req).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().sensor_name, "temp-01");
    }
}

// End of File
