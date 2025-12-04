use redpanda_sink::config::{Config, TimestampConfig};
use redpanda_sink::mapping::{extract_row, topic_matches, FieldValue};
use serial_test::serial;
use std::collections::BTreeMap;

/// Test configuration loading
#[tokio::test]
#[serial]
async fn test_config_loading() {
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
  - name: "test-pipeline"
    topic: "test-topic"
    table: "telemetry"
    data_type: "timeseries"
    timestamp:
      use_now: true
    tags:
      device_id: "$.device_id"
    fields:
      temperature: 
        path: "$.temperature"
        type: "float"
"#;

    let temp_file = std::env::temp_dir().join(format!("test-config-{}.yaml", std::process::id()));
    std::fs::write(&temp_file, config_str).unwrap();

    let original = std::env::var("REDPANDA_BROKERS").ok();
    std::env::remove_var("REDPANDA_BROKERS");

    let config = Config::load(&temp_file).unwrap();

    assert_eq!(config.redpanda.brokers, "localhost:9092");
    assert_eq!(config.redpanda.group_id, "test-group");
    assert_eq!(config.pipelines.len(), 1);
    assert_eq!(config.pipelines[0].name, "test-pipeline");
    assert_eq!(config.pipelines[0].data_type, "timeseries");

    if let Some(val) = original {
        std::env::set_var("REDPANDA_BROKERS", val);
    }
    std::fs::remove_file(&temp_file).ok();
}

/// Test environment variable override for Redpanda brokers
#[tokio::test]
#[serial]
async fn test_config_env_override() {
    let config_str = r#"
redpanda:
  brokers: "default:9092"

database:
  url: "postgres://default/test"

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

    let temp_file =
        std::env::temp_dir().join(format!("test-config-env-{}.yaml", std::process::id()));
    std::fs::write(&temp_file, config_str).unwrap();

    let original = std::env::var("REDPANDA_BROKERS").ok();

    std::env::set_var("REDPANDA_BROKERS", "override:9092");

    let config = Config::load(&temp_file).unwrap();
    assert_eq!(
        config.redpanda.brokers, "override:9092",
        "Environment variable override failed"
    );

    if let Some(val) = original {
        std::env::set_var("REDPANDA_BROKERS", val);
    } else {
        std::env::remove_var("REDPANDA_BROKERS");
    }

    std::fs::remove_file(&temp_file).ok();
}

/// Test message transformation to JSON format
#[tokio::test]
async fn test_message_transformation() {
    use redpanda_sink::config::{FieldConfig, Pipeline};

    let mut tags = BTreeMap::new();
    tags.insert("device_id".to_string(), "$.device_id".to_string());
    tags.insert("room".to_string(), "$.room".to_string());

    let mut fields = BTreeMap::new();
    fields.insert(
        "temperature_c".to_string(),
        FieldConfig {
            path: "$.temperature".to_string(),
            r#type: "float".to_string(),
            attributes: None,
        },
    );
    fields.insert(
        "power_w".to_string(),
        FieldConfig {
            path: "$.power".to_string(),
            r#type: "int".to_string(),
            attributes: None,
        },
    );
    fields.insert(
        "active".to_string(),
        FieldConfig {
            path: "$.active".to_string(),
            r#type: "bool".to_string(),
            attributes: None,
        },
    );

    let pipeline = Pipeline {
        name: "test".to_string(),
        topic: "test/topic".to_string(),
        table: "telemetry".to_string(),
        data_type: "timeseries".to_string(),
        upsert_key: None,
        timestamp: TimestampConfig {
            use_now: true,
            ..Default::default()
        },
        tags,
        fields,
        bit_flags: None,
        store_interval: None,
    };

    let payload = serde_json::json!({
        "device_id": "hp-01",
        "room": "utility",
        "temperature": 21.5,
        "power": 950,
        "active": true
    });

    let row = extract_row(&pipeline, "test/topic", payload.to_string().as_bytes()).unwrap();

    assert_eq!(row.tags.get("device_id").unwrap(), "hp-01");
    assert_eq!(row.tags.get("room").unwrap(), "utility");

    match row.fields.get("temperature_c").unwrap() {
        FieldValue::F64(v) => assert_eq!(*v, 21.5),
        _ => panic!("Expected F64"),
    }

    match row.fields.get("power_w").unwrap() {
        FieldValue::I64(v) => assert_eq!(*v, 950),
        _ => panic!("Expected I64"),
    }

    match row.fields.get("active").unwrap() {
        FieldValue::Bool(v) => assert!(*v),
        _ => panic!("Expected Bool"),
    }
}

/// Test pipeline matching with wildcards
#[tokio::test]
async fn test_pipeline_matching() {
    assert!(topic_matches(
        "home/heatpump/telemetry",
        "home/heatpump/telemetry"
    ));
    assert!(topic_matches("home/+/telemetry", "home/heatpump/telemetry"));
    assert!(topic_matches("home/+/telemetry", "home/sensor/telemetry"));
    assert!(!topic_matches(
        "home/+/telemetry",
        "home/heatpump/sensor/telemetry"
    ));
    assert!(topic_matches("home/#", "home/heatpump/telemetry"));
    assert!(topic_matches("home/#", "home/sensors/kitchen/state"));
    assert!(!topic_matches("home/#", "other/heatpump/telemetry"));
    assert!(!topic_matches(
        "home/heatpump/telemetry",
        "home/sensor/telemetry"
    ));
}

/// Test timestamp extraction with different formats
#[tokio::test]
async fn test_timestamp_extraction() {
    use redpanda_sink::config::{Pipeline, TimestampConfig};

    let pipeline_rfc3339 = Pipeline {
        name: "test".to_string(),
        topic: "test/topic".to_string(),
        table: "telemetry".to_string(),
        data_type: "timeseries".to_string(),
        upsert_key: None,
        timestamp: TimestampConfig {
            path: Some("$.timestamp".to_string()),
            format: "rfc3339".to_string(),
            use_now: false,
        },
        tags: BTreeMap::new(),
        fields: BTreeMap::new(),
        bit_flags: None,
        store_interval: None,
    };

    let payload = serde_json::json!({
        "timestamp": "2025-10-13T11:00:00Z"
    });

    let row = extract_row(
        &pipeline_rfc3339,
        "test/topic",
        payload.to_string().as_bytes(),
    )
    .unwrap();
    assert_eq!(
        row.ts.format("%Y-%m-%dT%H:%M:%S").to_string(),
        "2025-10-13T11:00:00"
    );
}

/// Test data type validation
#[tokio::test]
#[serial]
async fn test_data_type_validation() {
    let config_str = r#"
redpanda:
  brokers: "localhost:9092"

database:
  url: "postgres://localhost/test"

pipelines:
  - name: "static-pipeline"
    topic: "devices"
    table: "devices"
    data_type: "static"
    upsert_key: ["device_id"]
    timestamp:
      use_now: true
    tags:
      device_id: "$.device_id"
    fields: {}
"#;

    let temp_file =
        std::env::temp_dir().join(format!("test-config-dt-{}.yaml", std::process::id()));
    std::fs::write(&temp_file, config_str).unwrap();

    let config = Config::load(&temp_file).unwrap();
    assert_eq!(config.pipelines[0].data_type, "static");
    assert!(config.pipelines[0].upsert_key.is_some());

    std::fs::remove_file(&temp_file).ok();
}
