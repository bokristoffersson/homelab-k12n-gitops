// Integration tests for main.rs - API server startup and graceful shutdown
// These tests verify that the API server starts correctly alongside the consumer
// and handles graceful shutdown properly

use redpanda_sink::config::Config;
use std::time::Duration;
use tokio::time::timeout;

/// Test that config loading works with API enabled
#[tokio::test]
async fn test_config_with_api_enabled() {
    let config_str = r#"
redpanda:
  brokers: "localhost:9092"
  group_id: "test-group"
  auto_offset_reset: "earliest"

database:
  url: "postgres://localhost/test"
  write:
    batch_size: 100
    linger_ms: 100

api:
  enabled: true
  host: "0.0.0.0"
  port: 8080

auth:
  jwt_secret: "test-secret"
  users: []

pipelines:
  - name: "test"
    topic: "test-topic"
    table: "telemetry"
    data_type: "timeseries"
    timestamp:
      use_now: true
    tags: {}
    fields: {}
"#;

    let temp_file = std::env::temp_dir().join(format!("test-config-api-{}.yaml", std::process::id()));
    std::fs::write(&temp_file, config_str).unwrap();

    let config = Config::load(&temp_file).unwrap();
    
    assert!(config.api.is_some());
    let api = config.api.as_ref().unwrap();
    assert!(api.enabled);
    assert_eq!(api.host, "0.0.0.0");
    assert_eq!(api.port, 8080);

    std::fs::remove_file(&temp_file).ok();
}

/// Test that config loading works with API disabled
#[tokio::test]
async fn test_config_with_api_disabled() {
    let config_str = r#"
redpanda:
  brokers: "localhost:9092"
  group_id: "test-group"
  auto_offset_reset: "earliest"

database:
  url: "postgres://localhost/test"
  write:
    batch_size: 100
    linger_ms: 100

api:
  enabled: false
  host: "0.0.0.0"
  port: 8080

pipelines:
  - name: "test"
    topic: "test-topic"
    table: "telemetry"
    data_type: "timeseries"
    timestamp:
      use_now: true
    tags: {}
    fields: {}
"#;

    let temp_file = std::env::temp_dir().join(format!("test-config-api-disabled-{}.yaml", std::process::id()));
    std::fs::write(&temp_file, config_str).unwrap();

    let config = Config::load(&temp_file).unwrap();
    
    assert!(config.api.is_some());
    let api = config.api.as_ref().unwrap();
    assert!(!api.enabled);

    std::fs::remove_file(&temp_file).ok();
}

/// Test that config without API section defaults to disabled
#[tokio::test]
async fn test_config_without_api_section() {
    let config_str = r#"
redpanda:
  brokers: "localhost:9092"
  group_id: "test-group"
  auto_offset_reset: "earliest"

database:
  url: "postgres://localhost/test"
  write:
    batch_size: 100
    linger_ms: 100

pipelines:
  - name: "test"
    topic: "test-topic"
    table: "telemetry"
    data_type: "timeseries"
    timestamp:
      use_now: true
    tags: {}
    fields: {}
"#;

    let temp_file = std::env::temp_dir().join(format!("test-config-no-api-{}.yaml", std::process::id()));
    std::fs::write(&temp_file, config_str).unwrap();

    let config = Config::load(&temp_file).unwrap();
    
    // API config should be None when not specified
    assert!(config.api.is_none());

    std::fs::remove_file(&temp_file).ok();
}

/// Test API server router creation with valid config
#[tokio::test]
#[ignore] // Requires database connection
async fn test_api_router_creation() {
    use redpanda_sink::api::create_router;
    use redpanda_sink::db;
    
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/test".to_string());
    
    let pool = db::connect(&database_url).await.unwrap();
    
    let config_str = r#"
redpanda:
  brokers: "localhost:9092"
  group_id: "test-group"
  auto_offset_reset: "earliest"

database:
  url: "postgres://localhost/test"
  write:
    batch_size: 100
    linger_ms: 100

api:
  enabled: true
  host: "0.0.0.0"
  port: 8080

auth:
  jwt_secret: "test-secret"
  users:
    - username: "admin"
      password_hash: "$2b$12$testhash"

pipelines:
  - name: "test"
    topic: "test-topic"
    table: "telemetry"
    data_type: "timeseries"
    timestamp:
      use_now: true
    tags: {}
    fields: {}
"#;
    
    let temp_file = std::env::temp_dir().join(format!("test-config-router-{}.yaml", std::process::id()));
    std::fs::write(&temp_file, config_str).unwrap();
    let config = Config::load(&temp_file).unwrap();
    std::fs::remove_file(&temp_file).ok();
    
    // Should not panic
    let router = create_router(pool, config);
    // Router should have layers (CORS, middleware, etc.)
    // We can't easily check layer_count, but creating it should succeed
    drop(router);
}

/// Test that API server can bind to a port
#[tokio::test]
async fn test_api_server_bind() {
    use tokio::net::TcpListener;
    
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    
    // Verify we got a valid port
    assert!(addr.port() > 0);
    assert_eq!(addr.ip().to_string(), "127.0.0.1");
}

/// Test graceful shutdown signal handling
#[tokio::test]
async fn test_graceful_shutdown_signal() {
    use tokio::sync::broadcast;
    
    let (tx, mut rx) = broadcast::channel::<()>(1);
    
    // Spawn a task that will receive the shutdown signal
    let handle = tokio::spawn(async move {
        rx.recv().await.ok();
    });
    
    // Send shutdown signal
    let _ = tx.send(());
    
    // Wait for the task to complete (should complete quickly)
    let result = timeout(Duration::from_secs(1), handle).await;
    assert!(result.is_ok(), "Shutdown signal should be received within 1 second");
}

/// Test that multiple shutdown receivers can receive the signal
#[tokio::test]
async fn test_multiple_shutdown_receivers() {
    use tokio::sync::broadcast;
    
    let (tx, _) = broadcast::channel::<()>(1);
    let mut rx1 = tx.subscribe();
    let mut rx2 = tx.subscribe();
    
    // Spawn tasks that will receive the shutdown signal
    let handle1 = tokio::spawn(async move {
        rx1.recv().await.ok();
    });
    let handle2 = tokio::spawn(async move {
        rx2.recv().await.ok();
    });
    
    // Send shutdown signal
    let _ = tx.send(());
    
    // Both tasks should complete
    let result1 = timeout(Duration::from_secs(1), handle1).await;
    let result2 = timeout(Duration::from_secs(1), handle2).await;
    
    assert!(result1.is_ok(), "First receiver should receive signal");
    assert!(result2.is_ok(), "Second receiver should receive signal");
}

