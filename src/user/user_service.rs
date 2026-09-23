//! src/user/user_service.rs — Business logic for user registration and login.
//!

use std::str::FromStr;

#[cfg(test)]
use mockall::automock;
use validator::Validate;

use crate::auth::{password, token};
use crate::enums::Role;
use crate::error::AppError;
use crate::models::{AuthResponse, LoginRequest, RegisterRequest, UserInfo};

use super::DynUserRepo;

const TOKEN_TYPE_BEARER: &str = "Bearer";

#[cfg_attr(test, automock)]
#[async_trait::async_trait]
pub trait UserService: Send + Sync {
    async fn register_user(&self, req: RegisterRequest) -> Result<UserInfo, AppError>;
    async fn login_user(
        &self,
        req: LoginRequest,
        jwt_secret: &str,
    ) -> Result<AuthResponse, AppError>;
}

pub struct UserServiceImpl {
    user_repo: DynUserRepo,
}

impl UserServiceImpl {
    pub fn new(user_repo: DynUserRepo) -> Self {
        Self { user_repo }
    }
}

#[async_trait::async_trait]
impl UserService for UserServiceImpl {
    async fn register_user(&self, req: RegisterRequest) -> Result<UserInfo, AppError> {
        tracing::info!(email = %req.email, "register: start");

        req.validate()
            .map_err(|e| AppError::Validation(e.to_string()))?;

        let existing = self.user_repo.count_by_email(&req.email).await?;
        match existing {
            0 => {}
            _ => {
                tracing::warn!(email = %req.email, "register: user already exists");
                return Err(AppError::UserAlreadyExists);
            }
        }

        let password_hash = password::hash_password(&req.password)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        let user_id = uuid::Uuid::new_v4().to_string();

        self.user_repo
            .create(
                &user_id,
                &req.email,
                &password_hash,
                &Role::Operator.to_string(),
            )
            .await?;

        tracing::info!(email = %req.email, "register: completed");

        Ok(UserInfo {
            id: user_id,
            email: req.email,
            role: Role::Operator,
        })
    }

    async fn login_user(
        &self,
        req: LoginRequest,
        jwt_secret: &str,
    ) -> Result<AuthResponse, AppError> {
        tracing::info!(email = %req.email, "login: start");

        req.validate()
            .map_err(|e| AppError::Validation(e.to_string()))?;

        let user = match self.user_repo.find_by_email(&req.email).await? {
            Some(u) => u,
            None => {
                tracing::warn!(email = %req.email, "login: user not found");
                return Err(AppError::InvalidCredentials);
            }
        };

        match password::verify_password(&req.password, &user.password_hash) {
            Ok(()) => {}
            Err(_) => {
                tracing::warn!(email = %req.email, "login: invalid password");
                return Err(AppError::InvalidCredentials);
            }
        }

        let role = Role::from_str(&user.role).map_err(AppError::Internal)?;

        let token = token::encode_token(&user.id, &user.email, &role, jwt_secret)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        tracing::info!(email = %req.email, "login: completed");

        Ok(AuthResponse {
            token,
            token_type: TOKEN_TYPE_BEARER.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::User;
    use crate::user::user_repository::MockUserRepository;
    use std::sync::Arc;

    const STUB_JWT_SECRET: &str = "stub-secret-for-tests";
    const STUB_EMAIL: &str = "test@example.com";
    const STUB_PASSWORD: &str = "password123";

    fn make_user_service(mock: MockUserRepository) -> UserServiceImpl {
        UserServiceImpl::new(Arc::new(mock))
    }

    fn make_register_request() -> RegisterRequest {
        RegisterRequest {
            email: STUB_EMAIL.to_string(),
            password: STUB_PASSWORD.to_string(),
        }
    }

    fn make_login_request() -> LoginRequest {
        LoginRequest {
            email: STUB_EMAIL.to_string(),
            password: STUB_PASSWORD.to_string(),
        }
    }

    fn make_stored_user() -> User {
        let hash = password::hash_password(STUB_PASSWORD).unwrap();
        User {
            id: "user-1".to_string(),
            email: STUB_EMAIL.to_string(),
            password_hash: hash,
            role: "operator".to_string(),
            created_at: "2024-01-01 00:00:00".to_string(),
        }
    }

    #[tokio::test]
    async fn when_register_new_user_should_succeed() {
        let mut mock = MockUserRepository::new();
        mock.expect_count_by_email().returning(|_| Ok(0));
        mock.expect_create().returning(|_, _, _, _| Ok(()));

        let svc = make_user_service(mock);
        let result = svc.register_user(make_register_request()).await;

        assert!(result.is_ok());
        let info = result.unwrap();
        assert_eq!(info.email, STUB_EMAIL);
        assert_eq!(info.role, Role::Operator);
    }

    #[tokio::test]
    async fn when_register_duplicate_should_fail() {
        let mut mock = MockUserRepository::new();
        mock.expect_count_by_email().returning(|_| Ok(1));

        let svc = make_user_service(mock);
        let result = svc.register_user(make_register_request()).await;

        assert!(matches!(result, Err(AppError::UserAlreadyExists)));
    }

    #[tokio::test]
    async fn when_login_valid_should_return_token() {
        let mut mock = MockUserRepository::new();
        mock.expect_find_by_email()
            .returning(|_| Ok(Some(make_stored_user())));

        let svc = make_user_service(mock);
        let result = svc.login_user(make_login_request(), STUB_JWT_SECRET).await;

        assert!(result.is_ok());
        assert!(!result.unwrap().token.is_empty());
    }

    #[tokio::test]
    async fn when_login_nonexistent_should_fail() {
        let mut mock = MockUserRepository::new();
        mock.expect_find_by_email().returning(|_| Ok(None));

        let svc = make_user_service(mock);
        let result = svc.login_user(make_login_request(), STUB_JWT_SECRET).await;

        assert!(matches!(result, Err(AppError::InvalidCredentials)));
    }

    #[tokio::test]
    async fn when_login_wrong_password_should_fail() {
        let mut mock = MockUserRepository::new();
        mock.expect_find_by_email()
            .returning(|_| Ok(Some(make_stored_user())));

        let svc = make_user_service(mock);
        let req = LoginRequest {
            email: STUB_EMAIL.to_string(),
            password: "wrongpassword".to_string(),
        };
        let result = svc.login_user(req, STUB_JWT_SECRET).await;

        assert!(matches!(result, Err(AppError::InvalidCredentials)));
    }
}

// End of File
