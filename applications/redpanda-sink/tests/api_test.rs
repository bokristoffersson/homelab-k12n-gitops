// Integration tests for API endpoints
// These tests require a running database and proper test data

use axum::http::StatusCode;
use axum_test::TestServer;
use redpanda_sink::api::create_router;
use redpanda_sink::auth::hash_password;
use redpanda_sink::config::{
    ApiConfig, AuthConfig, Config, DbConfig, RedpandaConfig, User, WriteConfig,
};
use redpanda_sink::db;
use serde_json::json;

#[tokio::test]
#[ignore] // Requires database
async fn test_health_endpoint() {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/test".into());
    let pool = db::connect(&database_url).await.unwrap();

    let config = create_test_config();
    let app = create_router(pool, config);
    let server = TestServer::new(app).unwrap();

    let response = server.get("/health").await;
    response.assert_status(StatusCode::OK);
    response.assert_text("OK");
}

#[tokio::test]
#[ignore] // Requires database
async fn test_login_endpoint() {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/test".into());
    let pool = db::connect(&database_url).await.unwrap();

    let config = create_test_config();
    let app = create_router(pool, config);
    let server = TestServer::new(app).unwrap();

    // Test successful login
    let response = server
        .post("/api/v1/auth/login")
        .json(&json!({
            "username": "testuser",
            "password": "testpass"
        }))
        .await;

    response.assert_status(StatusCode::OK);
    let body: serde_json::Value = response.json();
    assert!(body.get("token").is_some());
    assert_eq!(body.get("username").unwrap().as_str().unwrap(), "testuser");

    // Test failed login
    let response = server
        .post("/api/v1/auth/login")
        .json(&json!({
            "username": "testuser",
            "password": "wrongpass"
        }))
        .await;

    response.assert_status(StatusCode::UNAUTHORIZED);
}

#[tokio::test]
#[ignore] // Requires database
async fn test_protected_endpoint_without_token() {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/test".into());
    let pool = db::connect(&database_url).await.unwrap();

    let config = create_test_config();
    let app = create_router(pool, config);
    let server = TestServer::new(app).unwrap();

    let response = server.get("/api/v1/energy/latest").await;
    response.assert_status(StatusCode::UNAUTHORIZED);
}

#[tokio::test]
#[ignore] // Requires database
async fn test_protected_endpoint_with_token() {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/test".into());
    let pool = db::connect(&database_url).await.unwrap();

    let config = create_test_config();
    let app = create_router(pool.clone(), config.clone());
    let server = TestServer::new(app).unwrap();

    // First login to get token
    let response = server
        .post("/api/v1/auth/login")
        .json(&json!({
            "username": "testuser",
            "password": "testpass"
        }))
        .await;

    response.assert_status(StatusCode::OK);
    let body: serde_json::Value = response.json();
    let token = body.get("token").unwrap().as_str().unwrap();

    // Use token to access protected endpoint
    let response = server
        .get("/api/v1/energy/latest")
        .add_header("Authorization", format!("Bearer {}", token))
        .await;

    // Should either succeed (if data exists) or return 500 (if no data)
    // But not 401 (unauthorized)
    let status = response.status_code();
    assert_ne!(status.as_u16(), 401);
}

#[tokio::test]
#[ignore] // Requires database
async fn test_login_endpoint_invalid_username() {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/test".into());
    let pool = db::connect(&database_url).await.unwrap();

    let config = create_test_config();
    let app = create_router(pool, config);
    let server = TestServer::new(app).unwrap();

    // Test with non-existent username
    let response = server
        .post("/api/v1/auth/login")
        .json(&json!({
            "username": "nonexistent",
            "password": "testpass"
        }))
        .await;

    response.assert_status(StatusCode::UNAUTHORIZED);
}

#[tokio::test]
#[ignore] // Requires database
async fn test_login_endpoint_missing_fields() {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/test".into());
    let pool = db::connect(&database_url).await.unwrap();

    let config = create_test_config();
    let app = create_router(pool, config);
    let server = TestServer::new(app).unwrap();

    // Test with missing password
    let response = server
        .post("/api/v1/auth/login")
        .json(&json!({
            "username": "testuser"
        }))
        .await;

    // Should return 400 Bad Request for invalid JSON
    assert!(response.status_code().as_u16() >= 400);
}

#[tokio::test]
#[ignore] // Requires database
async fn test_protected_endpoint_invalid_token() {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/test".into());
    let pool = db::connect(&database_url).await.unwrap();

    let config = create_test_config();
    let app = create_router(pool, config);
    let server = TestServer::new(app).unwrap();

    // Test with invalid token
    let response = server
        .get("/api/v1/energy/latest")
        .add_header("Authorization", "Bearer invalid-token-here")
        .await;

    response.assert_status(StatusCode::UNAUTHORIZED);
}

#[tokio::test]
#[ignore] // Requires database
async fn test_protected_endpoint_malformed_header() {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/test".into());
    let pool = db::connect(&database_url).await.unwrap();

    let config = create_test_config();
    let app = create_router(pool, config);
    let server = TestServer::new(app).unwrap();

    // Test with malformed Authorization header
    let response = server
        .get("/api/v1/energy/latest")
        .add_header("Authorization", "InvalidFormat token")
        .await;

    response.assert_status(StatusCode::UNAUTHORIZED);
}

#[tokio::test]
#[ignore] // Requires database
async fn test_energy_hourly_total_endpoint() {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/test".into());
    let pool = db::connect(&database_url).await.unwrap();

    let config = create_test_config();
    let app = create_router(pool.clone(), config.clone());
    let server = TestServer::new(app).unwrap();

    // Login first
    let login_response = server
        .post("/api/v1/auth/login")
        .json(&json!({
            "username": "testuser",
            "password": "testpass"
        }))
        .await;

    login_response.assert_status(StatusCode::OK);
    let login_body: serde_json::Value = login_response.json();
    let token = login_body.get("token").unwrap().as_str().unwrap();

    // Test hourly total endpoint
    let response = server
        .get("/api/v1/energy/hourly-total")
        .add_header("Authorization", format!("Bearer {}", token))
        .await;

    // Should either succeed (if data exists) or return 500 (if no data)
    // But not 401 (unauthorized)
    let status = response.status_code();
    assert_ne!(status.as_u16(), 401);

    if status.as_u16() == 200 {
        let body: serde_json::Value = response.json();
        assert!(body.get("total_kwh").is_some());
        assert!(body.get("hour_start").is_some());
        assert!(body.get("current_time").is_some());
    }
}

#[tokio::test]
#[ignore] // Requires database
async fn test_energy_history_endpoint() {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/test".into());
    let pool = db::connect(&database_url).await.unwrap();

    let config = create_test_config();
    let app = create_router(pool.clone(), config.clone());
    let server = TestServer::new(app).unwrap();

    // Login first
    let login_response = server
        .post("/api/v1/auth/login")
        .json(&json!({
            "username": "testuser",
            "password": "testpass"
        }))
        .await;

    login_response.assert_status(StatusCode::OK);
    let login_body: serde_json::Value = login_response.json();
    let token = login_body.get("token").unwrap().as_str().unwrap();

    // Test history endpoint with valid dates
    let from = chrono::Utc::now() - chrono::Duration::days(1);
    let to = chrono::Utc::now();

    let response = server
        .get(&format!(
            "/api/v1/energy/history?from={}&to={}",
            from.to_rfc3339(),
            to.to_rfc3339()
        ))
        .add_header("Authorization", format!("Bearer {}", token))
        .await;

    // Should either succeed (if data exists) or return 500 (if no data)
    // But not 401 (unauthorized)
    let status = response.status_code();
    assert_ne!(status.as_u16(), 401);

    if status.as_u16() == 200 {
        let body: serde_json::Value = response.json();
        assert!(body.is_array());
    }
}

#[tokio::test]
#[ignore] // Requires database
async fn test_energy_history_endpoint_missing_from() {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/test".into());
    let pool = db::connect(&database_url).await.unwrap();

    let config = create_test_config();
    let app = create_router(pool.clone(), config.clone());
    let server = TestServer::new(app).unwrap();

    // Login first
    let login_response = server
        .post("/api/v1/auth/login")
        .json(&json!({
            "username": "testuser",
            "password": "testpass"
        }))
        .await;

    login_response.assert_status(StatusCode::OK);
    let login_body: serde_json::Value = login_response.json();
    let token = login_body.get("token").unwrap().as_str().unwrap();

    // Test history endpoint without 'from' parameter (should fail)
    let response = server
        .get("/api/v1/energy/history")
        .add_header("Authorization", format!("Bearer {}", token))
        .await;

    response.assert_status(StatusCode::BAD_REQUEST); // Bad Request
}

#[tokio::test]
#[ignore] // Requires database
async fn test_energy_history_endpoint_invalid_date() {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/test".into());
    let pool = db::connect(&database_url).await.unwrap();

    let config = create_test_config();
    let app = create_router(pool.clone(), config.clone());
    let server = TestServer::new(app).unwrap();

    // Login first
    let login_response = server
        .post("/api/v1/auth/login")
        .json(&json!({
            "username": "testuser",
            "password": "testpass"
        }))
        .await;

    login_response.assert_status(StatusCode::OK);
    let login_body: serde_json::Value = login_response.json();
    let token = login_body.get("token").unwrap().as_str().unwrap();

    // Test history endpoint with invalid date format
    let response = server
        .get("/api/v1/energy/history?from=invalid-date")
        .add_header("Authorization", format!("Bearer {}", token))
        .await;

    response.assert_status(StatusCode::BAD_REQUEST); // Bad Request
}

#[tokio::test]
#[ignore] // Requires database
async fn test_heatpump_latest_endpoint() {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/test".into());
    let pool = db::connect(&database_url).await.unwrap();

    let config = create_test_config();
    let app = create_router(pool.clone(), config.clone());
    let server = TestServer::new(app).unwrap();

    // Login first
    let login_response = server
        .post("/api/v1/auth/login")
        .json(&json!({
            "username": "testuser",
            "password": "testpass"
        }))
        .await;

    login_response.assert_status(StatusCode::OK);
    let login_body: serde_json::Value = login_response.json();
    let token = login_body.get("token").unwrap().as_str().unwrap();

    // Test heatpump latest endpoint without device_id
    let response = server
        .get("/api/v1/heatpump/latest")
        .add_header("Authorization", format!("Bearer {}", token))
        .await;

    // Should either succeed (if data exists) or return 500 (if no data)
    // But not 401 (unauthorized)
    let status = response.status_code();
    assert_ne!(status.as_u16(), 401);

    if status.as_u16() == 200 {
        let body: serde_json::Value = response.json();
        assert!(body.get("ts").is_some());
    }
}

#[tokio::test]
#[ignore] // Requires database
async fn test_heatpump_latest_endpoint_with_device_id() {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/test".into());
    let pool = db::connect(&database_url).await.unwrap();

    let config = create_test_config();
    let app = create_router(pool.clone(), config.clone());
    let server = TestServer::new(app).unwrap();

    // Login first
    let login_response = server
        .post("/api/v1/auth/login")
        .json(&json!({
            "username": "testuser",
            "password": "testpass"
        }))
        .await;

    login_response.assert_status(StatusCode::OK);
    let login_body: serde_json::Value = login_response.json();
    let token = login_body.get("token").unwrap().as_str().unwrap();

    // Test heatpump latest endpoint with device_id parameter
    // device_id filtering is now supported
    let response = server
        .get("/api/v1/heatpump/latest?device_id=test-device")
        .add_header("Authorization", format!("Bearer {}", token))
        .await;

    // Should either succeed (if data exists) or return 404/500 (if no data)
    // But not 401 (unauthorized)
    let status = response.status_code();
    assert_ne!(status.as_u16(), 401);
}

#[tokio::test]
#[ignore] // Requires database
async fn test_token_expires_in_field() {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/test".into());
    let pool = db::connect(&database_url).await.unwrap();

    let config = create_test_config();
    let app = create_router(pool, config);
    let server = TestServer::new(app).unwrap();

    let response = server
        .post("/api/v1/auth/login")
        .json(&json!({
            "username": "testuser",
            "password": "testpass"
        }))
        .await;

    response.assert_status(StatusCode::OK);
    let body: serde_json::Value = response.json();

    // Verify expires_in is present and is a number
    assert!(body.get("expires_in").is_some());
    let expires_in = body.get("expires_in").unwrap().as_u64().unwrap();
    // Should be 24 hours * 3600 seconds = 86400
    assert_eq!(expires_in, 86400);
}

fn create_test_config() -> Config {
    let password_hash = hash_password("testpass").unwrap();

    Config {
        redpanda: RedpandaConfig {
            brokers: "localhost:9092".into(),
            group_id: "test".into(),
            auto_offset_reset: "earliest".into(),
        },
        database: DbConfig {
            url: "postgres://postgres:postgres@localhost:5432/test".into(),
            write: WriteConfig {
                batch_size: 100,
                linger_ms: 100,
            },
        },
        pipelines: vec![],
        api: Some(ApiConfig {
            enabled: true,
            host: "0.0.0.0".into(),
            port: 8080,
        }),
        auth: Some(AuthConfig {
            jwt_secret: "test-secret-key-for-testing-only".into(),
            jwt_expiry_hours: 24,
            users: vec![User {
                username: "testuser".into(),
                password_hash,
            }],
        }),
    }
}
