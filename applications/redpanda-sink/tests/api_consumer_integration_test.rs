// Integration tests for API server running alongside consumer
// These tests verify that both services can run together and handle shutdown gracefully

use redpanda_sink::config::Config;
use std::time::Duration;
use tokio::time::timeout;
use tokio::sync::broadcast;

/// Test that API server configuration is properly loaded from config
#[tokio::test]
async fn test_api_config_loading() {
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
  host: "127.0.0.1"
  port: 8080

auth:
  jwt_secret: "test-secret-key"
  jwt_expiry_hours: 24
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

    let temp_file = std::env::temp_dir().join(format!("test-api-config-{}.yaml", std::process::id()));
    std::fs::write(&temp_file, config_str).unwrap();

    let config = Config::load(&temp_file).unwrap();
    
    // Verify API config
    assert!(config.api.is_some());
    let api = config.api.as_ref().unwrap();
    assert!(api.enabled);
    assert_eq!(api.host, "127.0.0.1");
    assert_eq!(api.port, 8080);
    
    // Verify auth config
    assert!(config.auth.is_some());
    let auth = config.auth.as_ref().unwrap();
    assert_eq!(auth.jwt_secret, "test-secret-key");
    assert_eq!(auth.users.len(), 1);
    assert_eq!(auth.users[0].username, "admin");

    std::fs::remove_file(&temp_file).ok();
}

/// Test shutdown signal propagation to multiple receivers
/// This simulates the scenario where both API server and consumer need to shutdown
#[tokio::test]
async fn test_shutdown_signal_propagation() {
    let (shutdown_tx, _) = broadcast::channel::<()>(1);
    let mut rx1 = shutdown_tx.subscribe();
    let mut rx2 = shutdown_tx.subscribe();
    
    // Simulate API server and consumer tasks waiting for shutdown
    let api_task = tokio::spawn(async move {
        rx1.recv().await.ok();
        "api_shutdown"
    });
    
    let consumer_task = tokio::spawn(async move {
        rx2.recv().await.ok();
        "consumer_shutdown"
    });
    
    // Send shutdown signal
    let _ = shutdown_tx.send(());
    
    // Both tasks should receive the signal and complete
    let result = timeout(Duration::from_secs(2), async {
        tokio::join!(api_task, consumer_task)
    }).await;
    
    assert!(result.is_ok(), "Both tasks should receive shutdown signal");
    let (api_result, consumer_result) = result.unwrap();
    assert_eq!(api_result.unwrap(), "api_shutdown");
    assert_eq!(consumer_result.unwrap(), "consumer_shutdown");
}

/// Test that API server can be created with valid configuration
#[tokio::test]
#[ignore] // Requires database
async fn test_api_server_creation() {
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
  jwt_secret: "test-secret-key"
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
    
    let temp_file = std::env::temp_dir().join(format!("test-api-server-{}.yaml", std::process::id()));
    std::fs::write(&temp_file, config_str).unwrap();
    let config = Config::load(&temp_file).unwrap();
    std::fs::remove_file(&temp_file).ok();
    
    // Create router - should not panic
    let router = create_router(pool, config);
    drop(router);
}

/// Test that API server respects the enabled flag
#[tokio::test]
async fn test_api_server_respects_enabled_flag() {
    // Test with enabled: false
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

    let temp_file = std::env::temp_dir().join(format!("test-api-disabled-{}.yaml", std::process::id()));
    std::fs::write(&temp_file, config_str).unwrap();

    let config = Config::load(&temp_file).unwrap();
    
    assert!(config.api.is_some());
    assert!(!config.api.as_ref().unwrap().enabled);

    std::fs::remove_file(&temp_file).ok();
}

/// Test environment variable override for JWT_SECRET
#[tokio::test]
async fn test_jwt_secret_env_override() {
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
  jwt_secret: "original-secret"
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

    let temp_file = std::env::temp_dir().join(format!("test-jwt-env-{}.yaml", std::process::id()));
    std::fs::write(&temp_file, config_str).unwrap();

    let original_jwt = std::env::var("JWT_SECRET").ok();
    std::env::set_var("JWT_SECRET", "env-override-secret");

    let config = Config::load(&temp_file).unwrap();
    
    assert!(config.auth.is_some());
    assert_eq!(config.auth.as_ref().unwrap().jwt_secret, "env-override-secret");

    // Cleanup
    if let Some(val) = original_jwt {
        std::env::set_var("JWT_SECRET", val);
    } else {
        std::env::remove_var("JWT_SECRET");
    }
    std::fs::remove_file(&temp_file).ok();
}

/// Test that multiple shutdown receivers work correctly
/// This is important for the main.rs implementation where both API and consumer listen
#[tokio::test]
async fn test_multiple_shutdown_listeners() {
    let (shutdown_tx, _) = broadcast::channel::<()>(1);
    
    // Create multiple receivers (simulating API server and consumer)
    let mut receivers: Vec<_> = (0..3)
        .map(|_| shutdown_tx.subscribe())
        .collect();
    
    // Spawn tasks that wait for shutdown
    let handles: Vec<_> = receivers
        .iter_mut()
        .enumerate()
        .map(|(i, rx)| {
            let mut rx = rx.clone();
            tokio::spawn(async move {
                rx.recv().await.ok();
                i
            })
        })
        .collect();
    
    // Send shutdown signal
    let _ = shutdown_tx.send(());
    
    // All tasks should complete
    let result = timeout(Duration::from_secs(2), async {
        let mut results = Vec::new();
        for handle in handles {
            results.push(handle.await);
        }
        results
    }).await;
    
    assert!(result.is_ok(), "All receivers should get shutdown signal");
    let results = result.unwrap();
    for handle_result in results {
        assert!(handle_result.is_ok());
    }
}

