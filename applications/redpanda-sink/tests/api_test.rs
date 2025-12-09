// Integration tests for API endpoints
// These tests require a running database and proper test data

use redpanda_sink::api::create_router;
use redpanda_sink::auth::hash_password;
use redpanda_sink::config::{Config, ApiConfig, AuthConfig, User, RedpandaConfig, DbConfig, WriteConfig};
use redpanda_sink::db;
use axum_test::TestServer;
use serde_json::json;

#[tokio::test]
#[ignore] // Requires database
async fn test_health_endpoint() {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/test".into());
    let pool = db::connect(&database_url).await.unwrap();
    
    let config = create_test_config();
    let app = create_router(pool, config);
    let server = TestServer::new(app.into_make_service()).unwrap();
    
    let response = server.get("/health").await;
    response.assert_status(200);
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
    let server = TestServer::new(app.into_make_service()).unwrap();
    
    // Test successful login
    let response = server
        .post("/api/v1/auth/login")
        .json(&json!({
            "username": "testuser",
            "password": "testpass"
        }))
        .await;
    
    response.assert_status(200);
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
    
    response.assert_status(401);
}

#[tokio::test]
#[ignore] // Requires database
async fn test_protected_endpoint_without_token() {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/test".into());
    let pool = db::connect(&database_url).await.unwrap();
    
    let config = create_test_config();
    let app = create_router(pool, config);
    let server = TestServer::new(app.into_make_service()).unwrap();
    
    let response = server.get("/api/v1/energy/latest").await;
    response.assert_status(401);
}

#[tokio::test]
#[ignore] // Requires database
async fn test_protected_endpoint_with_token() {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/test".into());
    let pool = db::connect(&database_url).await.unwrap();
    
    let config = create_test_config();
    let app = create_router(pool.clone(), config.clone());
    let server = TestServer::new(app.into_make_service()).unwrap();
    
    // First login to get token
    let response = server
        .post("/api/v1/auth/login")
        .json(&json!({
            "username": "testuser",
            "password": "testpass"
        }))
        .await;
    
    response.assert_status(200);
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
        api: ApiConfig {
            enabled: true,
            host: "0.0.0.0".into(),
            port: 8080,
        },
        auth: AuthConfig {
            jwt_secret: "test-secret-key-for-testing-only".into(),
            jwt_expiry_hours: 24,
            users: vec![User {
                username: "testuser".into(),
                password_hash,
            }],
        },
    }
}


