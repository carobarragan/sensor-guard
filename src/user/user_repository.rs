//! src/user/user_repository.rs — Data access for the users table.
//!

#[cfg(test)]
use mockall::automock;
use sqlx::SqlitePool;

use crate::error::AppError;
use crate::models::User;

#[cfg_attr(test, automock)]
#[async_trait::async_trait]
pub trait UserRepository: Send + Sync {
    async fn count_by_email(&self, email: &str) -> Result<i64, AppError>;
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, AppError>;
    async fn create(
        &self,
        id: &str,
        email: &str,
        password_hash: &str,
        role: &str,
    ) -> Result<(), AppError>;
}

pub struct SqliteUserRepository {
    pool: SqlitePool,
}

impl SqliteUserRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl UserRepository for SqliteUserRepository {
    async fn count_by_email(&self, email: &str) -> Result<i64, AppError> {
        let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users WHERE email = ?")
            .bind(email)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        Ok(count)
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>, AppError> {
        let user = sqlx::query_as::<_, User>(
            "SELECT id, email, password_hash, role, created_at FROM users WHERE email = ?",
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
        Ok(user)
    }

    async fn create(
        &self,
        id: &str,
        email: &str,
        password_hash: &str,
        role: &str,
    ) -> Result<(), AppError> {
        sqlx::query("INSERT INTO users (id, email, password_hash, role) VALUES (?, ?, ?, ?)")
            .bind(id)
            .bind(email)
            .bind(password_hash)
            .bind(role)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        Ok(())
    }
}

// End of File
