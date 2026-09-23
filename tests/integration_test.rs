//! tests/integration_test.rs — Integration tests for auth and authorization flows.
//!

use std::sync::Arc;

use reqwest::StatusCode;
use sensor_guard::reading::reading_repository::SqliteReadingRepository;
use sensor_guard::reading::reading_service::ReadingServiceImpl;
use sensor_guard::user::user_repository::SqliteUserRepository;
use sensor_guard::user::user_service::UserServiceImpl;
use sensor_guard::{build_router_without_rate_limit, AppState};
use serde_json::{json, Value};

const STUB_JWT_SECRET: &str = "integration-test-secret-not-for-production";

/// Spin up a test server on a random port with an in-memory SQLite database.
async fn make_test_server() -> String {
    let pool = sensor_guard::db::init_db("sqlite::memory:")
        .await
        .expect("Failed to init test DB");

    let user_repo = Arc::new(SqliteUserRepository::new(pool.clone()));
    let reading_repo = Arc::new(SqliteReadingRepository::new(pool));

    let state = AppState {
        jwt_secret: STUB_JWT_SECRET.to_string(),
        user_service: Arc::new(UserServiceImpl::new(user_repo)),
        reading_service: Arc::new(ReadingServiceImpl::new(reading_repo)),
    };

    let app = build_router_without_rate_limit(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind test listener");
    let addr = listener.local_addr().unwrap();
    let base_url = format!("http://{}", addr);

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    base_url
}

/// Helper: register a user and return the response.
async fn register_user(
    client: &reqwest::Client,
    base: &str,
    email: &str,
    password: &str,
) -> reqwest::Response {
    client
        .post(format!("{base}/auth/register"))
        .json(&json!({
            "email": email,
            "password": password,
        }))
        .send()
        .await
        .unwrap()
}

/// Helper: login and return the JWT token.
async fn login_user(client: &reqwest::Client, base: &str, email: &str, password: &str) -> String {
    let resp = client
        .post(format!("{base}/auth/login"))
        .json(&json!({
            "email": email,
            "password": password,
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = resp.json().await.unwrap();
    body["token"].as_str().unwrap().to_string()
}

/// Helper: seed an admin user directly in the DB.
async fn seed_admin(pool: &sqlx::SqlitePool, email: &str, password: &str) {
    use argon2::{
        password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
        Argon2,
    };
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .unwrap()
        .to_string();
    let id = uuid::Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO users (id, email, password_hash, role) VALUES (?, ?, ?, 'admin')")
        .bind(&id)
        .bind(email)
        .bind(&hash)
        .execute(pool)
        .await
        .unwrap();
}

/// Spawn a test server that also has a pre-seeded admin user.
async fn make_test_server_with_admin() -> (String, sqlx::SqlitePool) {
    let pool = sensor_guard::db::init_db("sqlite::memory:")
        .await
        .expect("Failed to init test DB");

    seed_admin(&pool, "admin@test.com", "adminpass123").await;

    let user_repo = Arc::new(SqliteUserRepository::new(pool.clone()));
    let reading_repo = Arc::new(SqliteReadingRepository::new(pool.clone()));

    let state = AppState {
        jwt_secret: STUB_JWT_SECRET.to_string(),
        user_service: Arc::new(UserServiceImpl::new(user_repo)),
        reading_service: Arc::new(ReadingServiceImpl::new(reading_repo)),
    };

    let app = build_router_without_rate_limit(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind test listener");
    let addr = listener.local_addr().unwrap();
    let base_url = format!("http://{}", addr);

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    (base_url, pool)
}

// ── Auth tests ───────────────────────────────────────────────────

#[tokio::test]
async fn when_registering_valid_user_should_succeed() {
    let base = make_test_server().await;
    let client = reqwest::Client::new();

    let resp = register_user(&client, &base, "new@example.com", "password123").await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["email"], "new@example.com");
    assert_eq!(body["role"], "operator");
}

#[tokio::test]
async fn when_registering_duplicate_email_should_fail() {
    let base = make_test_server().await;
    let client = reqwest::Client::new();

    register_user(&client, &base, "dup@example.com", "password123").await;
    let resp = register_user(&client, &base, "dup@example.com", "password456").await;
    assert_eq!(resp.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn when_registering_with_short_password_should_fail() {
    let base = make_test_server().await;
    let client = reqwest::Client::new();

    let resp = register_user(&client, &base, "short@example.com", "1234567").await;
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn when_registering_with_invalid_email_should_fail() {
    let base = make_test_server().await;
    let client = reqwest::Client::new();

    let resp = register_user(&client, &base, "not-an-email", "password123").await;
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn when_logging_in_with_correct_credentials_should_return_token() {
    let base = make_test_server().await;
    let client = reqwest::Client::new();

    register_user(&client, &base, "login@example.com", "password123").await;
    let token = login_user(&client, &base, "login@example.com", "password123").await;
    assert!(!token.is_empty());
}

#[tokio::test]
async fn when_logging_in_with_wrong_password_should_fail() {
    let base = make_test_server().await;
    let client = reqwest::Client::new();

    register_user(&client, &base, "wrongpw@example.com", "password123").await;

    let resp = client
        .post(format!("{base}/auth/login"))
        .json(&json!({
            "email": "wrongpw@example.com",
            "password": "wrongpassword",
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"], "Invalid email or password");
}

#[tokio::test]
async fn when_logging_in_with_nonexistent_user_should_return_same_error() {
    let base = make_test_server().await;
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base}/auth/login"))
        .json(&json!({
            "email": "nobody@example.com",
            "password": "password123",
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"], "Invalid email or password");
}

// ── Authorization tests ──────────────────────────────────────────

#[tokio::test]
async fn when_accessing_sensors_without_token_should_fail() {
    let base = make_test_server().await;
    let client = reqwest::Client::new();

    let resp = client.get(format!("{base}/sensors")).send().await.unwrap();

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn when_operator_lists_sensors_should_succeed() {
    let base = make_test_server().await;
    let client = reqwest::Client::new();

    register_user(&client, &base, "op@example.com", "password123").await;
    let token = login_user(&client, &base, "op@example.com", "password123").await;

    let resp = client
        .get(format!("{base}/sensors"))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn when_operator_creates_reading_should_be_forbidden() {
    let base = make_test_server().await;
    let client = reqwest::Client::new();

    register_user(&client, &base, "op2@example.com", "password123").await;
    let token = login_user(&client, &base, "op2@example.com", "password123").await;

    let resp = client
        .post(format!("{base}/sensors"))
        .header("Authorization", format!("Bearer {token}"))
        .json(&json!({
            "sensor_name": "temp-01",
            "value": 25.5,
            "unit": "°C",
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn when_admin_creates_reading_should_succeed() {
    let (base, _pool) = make_test_server_with_admin().await;
    let client = reqwest::Client::new();

    let token = login_user(&client, &base, "admin@test.com", "adminpass123").await;

    let resp = client
        .post(format!("{base}/sensors"))
        .header("Authorization", format!("Bearer {token}"))
        .json(&json!({
            "sensor_name": "pressure-01",
            "value": 101.3,
            "unit": "kPa",
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::CREATED);

    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["sensor_name"], "pressure-01");
    assert_eq!(body["unit"], "kPa");
}

#[tokio::test]
async fn when_admin_creates_and_gets_reading_should_match() {
    let (base, _pool) = make_test_server_with_admin().await;
    let client = reqwest::Client::new();

    let token = login_user(&client, &base, "admin@test.com", "adminpass123").await;

    let resp = client
        .post(format!("{base}/sensors"))
        .header("Authorization", format!("Bearer {token}"))
        .json(&json!({
            "sensor_name": "vibration-01",
            "value": 0.42,
            "unit": "mm/s",
        }))
        .send()
        .await
        .unwrap();
    let created: Value = resp.json().await.unwrap();
    let id = created["id"].as_str().unwrap();

    let resp = client
        .get(format!("{base}/sensors/{id}"))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let fetched: Value = resp.json().await.unwrap();
    assert_eq!(fetched["sensor_name"], "vibration-01");
}

#[tokio::test]
async fn when_getting_nonexistent_reading_should_return_404() {
    let base = make_test_server().await;
    let client = reqwest::Client::new();

    register_user(&client, &base, "user404@example.com", "password123").await;
    let token = login_user(&client, &base, "user404@example.com", "password123").await;

    let resp = client
        .get(format!("{base}/sensors/nonexistent-id"))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn when_creating_reading_with_invalid_input_should_fail() {
    let (base, _pool) = make_test_server_with_admin().await;
    let client = reqwest::Client::new();

    let token = login_user(&client, &base, "admin@test.com", "adminpass123").await;

    let resp = client
        .post(format!("{base}/sensors"))
        .header("Authorization", format!("Bearer {token}"))
        .json(&json!({
            "sensor_name": "",
            "value": 1.0,
            "unit": "V",
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

// End of File
