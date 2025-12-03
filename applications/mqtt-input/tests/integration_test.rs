use mqtt_input::config::{Config, TimestampConfig};
use mqtt_input::ingest::Ingestor;
use mqtt_input::mapping::FieldValue;
use serial_test::serial;
use std::collections::BTreeMap;

/// Test configuration loading
#[tokio::test]
#[serial]
async fn test_config_loading() {
    let config_str = r#"
mqtt:
  host: "localhost"
  port: 1883
  client_id: "test-client"
  username: "test"
  password: "test"
  keep_alive_secs: 30
  clean_session: true
  protocol: "v5"

redpanda:
  brokers: "localhost:9092"

pipelines:
  - name: "test-pipeline"
    topic: "test/topic"
    qos: 1
    redpanda_topic: "test-redpanda-topic"
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

    // Ensure REDPANDA_BROKERS is not set for this test
    let original = std::env::var("REDPANDA_BROKERS").ok();
    std::env::remove_var("REDPANDA_BROKERS");

    let config = Config::load(&temp_file).unwrap();

    assert_eq!(config.mqtt.host, "localhost");
    assert_eq!(config.mqtt.port, 1883);
    // Brokers from config file
    assert_eq!(config.redpanda.brokers, "localhost:9092");

    // Restore original if it existed
    if let Some(val) = original {
        std::env::set_var("REDPANDA_BROKERS", val);
    }
    assert_eq!(config.pipelines.len(), 1);
    assert_eq!(config.pipelines[0].name, "test-pipeline");
    assert_eq!(config.pipelines[0].redpanda_topic, "test-redpanda-topic");

    std::fs::remove_file(&temp_file).ok();
}

/// Test environment variable override for Redpanda brokers
#[tokio::test]
#[serial]
async fn test_config_env_override() {
    let config_str = r#"
mqtt:
  host: "localhost"
  port: 1883
  client_id: "test-client"

redpanda:
  brokers: "default:9092"

pipelines:
  - name: "test"
    topic: "test/topic"
    qos: 1
    redpanda_topic: "test-topic"
    timestamp:
      use_now: true
"#;

    let temp_file =
        std::env::temp_dir().join(format!("test-config-env-{}.yaml", std::process::id()));
    std::fs::write(&temp_file, config_str).unwrap();

    // Save original value if it exists
    let original = std::env::var("REDPANDA_BROKERS").ok();

    // Set the environment variable
    std::env::set_var("REDPANDA_BROKERS", "override:9092");

    // Verify it's set before loading config
    assert_eq!(std::env::var("REDPANDA_BROKERS").unwrap(), "override:9092");

    let config = Config::load(&temp_file).unwrap();
    assert_eq!(
        config.redpanda.brokers,
        "override:9092",
        "Environment variable override failed. Config has: {}, Env var is: {:?}",
        config.redpanda.brokers,
        std::env::var("REDPANDA_BROKERS").ok()
    );

    // Restore original value or remove
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
    use mqtt_input::config::{FieldConfig, Pipeline};
    use mqtt_input::mapping::{extract_row, FieldValue};
    use serde_json::json;

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
        qos: 1,
        redpanda_topic: "test-topic".to_string(),
        timestamp: TimestampConfig {
            use_now: true,
            ..Default::default()
        },
        tags,
        fields,
        bit_flags: None,
        store_interval: None,
    };

    let payload = json!({
        "device_id": "hp-01",
        "room": "utility",
        "temperature": 21.5,
        "power": 950,
        "active": true
    });

    let row = extract_row(&pipeline, "test/topic", payload.to_string().as_bytes()).unwrap();

    // Verify extracted data
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
        FieldValue::Bool(v) => assert_eq!(*v, true),
        _ => panic!("Expected Bool"),
    }
}

/// Test pipeline matching with wildcards
#[tokio::test]
async fn test_pipeline_matching() {
    use mqtt_input::mapping::topic_matches;

    // Test exact match
    assert!(topic_matches(
        "home/heatpump/telemetry",
        "home/heatpump/telemetry"
    ));

    // Test single-level wildcard
    assert!(topic_matches("home/+/telemetry", "home/heatpump/telemetry"));
    assert!(topic_matches("home/+/telemetry", "home/sensor/telemetry"));
    assert!(!topic_matches(
        "home/+/telemetry",
        "home/heatpump/sensor/telemetry"
    ));

    // Test multi-level wildcard
    assert!(topic_matches("home/#", "home/heatpump/telemetry"));
    assert!(topic_matches("home/#", "home/sensors/kitchen/state"));
    assert!(!topic_matches("home/#", "other/heatpump/telemetry"));

    // Test no match
    assert!(!topic_matches(
        "home/heatpump/telemetry",
        "home/sensor/telemetry"
    ));
}

/// Test interval throttling logic
#[tokio::test]
async fn test_interval_throttling() {
    use chrono::Utc;
    use mqtt_input::redpanda::create_producer;

    // Create a producer for the ingestor
    // Note: This test requires a running Redpanda instance or we skip it
    // In a real scenario, you'd use testcontainers or mocks
    let brokers = "localhost:9092";

    if let Ok(producer) = create_producer(brokers).await {
        let ingestor = Ingestor::new(producer);

        let now = Utc::now();
        let pipeline_name = "test-pipeline";

        // First call should allow storage
        let should_store1 = ingestor
            .should_store(pipeline_name, &now, "MINUTE")
            .unwrap();
        assert!(should_store1);

        // Immediate second call should be throttled
        let should_store2 = ingestor
            .should_store(pipeline_name, &now, "MINUTE")
            .unwrap();
        assert!(!should_store2);

        // After a minute, should allow again
        let later = now + chrono::Duration::minutes(1);
        let should_store3 = ingestor
            .should_store(pipeline_name, &later, "MINUTE")
            .unwrap();
        assert!(should_store3);
    } else {
        // Skip test if Redpanda is not available - this is expected in CI without testcontainers
        println!("Skipping test - Redpanda not available (expected in CI)");
    }
}

/// Test bit flag parsing
#[tokio::test]
async fn test_bit_flag_extraction() {
    use mqtt_input::config::{BitFlagConfig, Pipeline};
    use mqtt_input::mapping::extract_row;
    use serde_json::json;

    let mut bit_flags = BTreeMap::new();
    bit_flags.insert(0, "compressor_on".to_string());
    bit_flags.insert(1, "heating_mode".to_string());
    bit_flags.insert(2, "hot_water_mode".to_string());
    bit_flags.insert(4, "circulation_pump".to_string());

    let pipeline = Pipeline {
        name: "test".to_string(),
        topic: "test/topic".to_string(),
        qos: 1,
        redpanda_topic: "test-topic".to_string(),
        timestamp: TimestampConfig {
            use_now: true,
            ..Default::default()
        },
        tags: BTreeMap::new(),
        fields: BTreeMap::new(),
        bit_flags: Some(vec![BitFlagConfig {
            source_path: "$.status_byte".to_string(),
            flags: bit_flags,
        }]),
        store_interval: None,
    };

    let payload = json!({
        "status_byte": 21  // 0b00010101 - bits 0, 2, 4 set
    });

    let row = extract_row(&pipeline, "test/topic", payload.to_string().as_bytes()).unwrap();

    // Check bit flags
    match row.fields.get("compressor_on").unwrap() {
        FieldValue::Bool(v) => assert_eq!(*v, true),
        _ => panic!("Expected Bool"),
    }

    match row.fields.get("heating_mode").unwrap() {
        FieldValue::Bool(v) => assert_eq!(*v, false),
        _ => panic!("Expected Bool"),
    }

    match row.fields.get("hot_water_mode").unwrap() {
        FieldValue::Bool(v) => assert_eq!(*v, true),
        _ => panic!("Expected Bool"),
    }

    match row.fields.get("circulation_pump").unwrap() {
        FieldValue::Bool(v) => assert_eq!(*v, true),
        _ => panic!("Expected Bool"),
    }
}

/// Test nested field extraction
#[tokio::test]
async fn test_nested_field_extraction() {
    use mqtt_input::config::{FieldConfig, Pipeline};
    use mqtt_input::mapping::extract_row;
    use serde_json::json;

    let mut consumption_attrs = BTreeMap::new();
    consumption_attrs.insert("total".to_string(), "consumption_total_w".to_string());
    consumption_attrs.insert("L1".to_string(), "consumption_l1_w".to_string());
    consumption_attrs.insert("L2".to_string(), "consumption_l2_w".to_string());

    let mut fields = BTreeMap::new();
    fields.insert(
        "activeActualConsumption".to_string(),
        FieldConfig {
            path: "$.activeActualConsumption".to_string(),
            r#type: "nested".to_string(),
            attributes: Some(consumption_attrs),
        },
    );

    let pipeline = Pipeline {
        name: "test".to_string(),
        topic: "test/topic".to_string(),
        qos: 1,
        redpanda_topic: "test-topic".to_string(),
        timestamp: TimestampConfig {
            use_now: true,
            ..Default::default()
        },
        tags: BTreeMap::new(),
        fields,
        bit_flags: None,
        store_interval: None,
    };

    let payload = json!({
        "activeActualConsumption": {
            "total": 622,
            "L1": 299,
            "L2": 194
        }
    });

    let row = extract_row(&pipeline, "test/topic", payload.to_string().as_bytes()).unwrap();

    // Check nested fields
    match row.fields.get("consumption_total_w").unwrap() {
        FieldValue::F64(v) => assert_eq!(*v, 622.0),
        _ => panic!("Expected F64"),
    }

    match row.fields.get("consumption_l1_w").unwrap() {
        FieldValue::F64(v) => assert_eq!(*v, 299.0),
        _ => panic!("Expected F64"),
    }

    match row.fields.get("consumption_l2_w").unwrap() {
        FieldValue::F64(v) => assert_eq!(*v, 194.0),
        _ => panic!("Expected F64"),
    }
}

/// Test timestamp extraction with different formats
#[tokio::test]
async fn test_timestamp_extraction() {
    use mqtt_input::config::{Pipeline, TimestampConfig};
    use mqtt_input::mapping::extract_row;
    use serde_json::json;

    // Test RFC3339 format
    let pipeline_rfc3339 = Pipeline {
        name: "test".to_string(),
        topic: "test/topic".to_string(),
        qos: 1,
        redpanda_topic: "test-topic".to_string(),
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

    let payload = json!({
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

    // Test unix_ms format
    let pipeline_unix_ms = Pipeline {
        name: "test".to_string(),
        topic: "test/topic".to_string(),
        qos: 1,
        redpanda_topic: "test-topic".to_string(),
        timestamp: TimestampConfig {
            path: Some("$.timestamp".to_string()),
            format: "unix_ms".to_string(),
            use_now: false,
        },
        tags: BTreeMap::new(),
        fields: BTreeMap::new(),
        bit_flags: None,
        store_interval: None,
    };

    // Use a known timestamp: 2024-01-15T10:30:00Z = 1705314600000 milliseconds
    let payload_unix = json!({
        "timestamp": 1705314600000i64
    });

    let row_unix = extract_row(
        &pipeline_unix_ms,
        "test/topic",
        payload_unix.to_string().as_bytes(),
    )
    .unwrap();
    // Verify the timestamp is correct (UTC)
    let expected_ts = chrono::DateTime::parse_from_rfc3339("2024-01-15T10:30:00Z")
        .unwrap()
        .with_timezone(&chrono::Utc);
    // Allow small difference for rounding
    let diff = (row_unix.ts - expected_ts).num_seconds().abs();
    assert!(
        diff <= 1,
        "Timestamp difference too large: {} seconds. Expected: {:?}, Got: {:?}",
        diff,
        expected_ts,
        row_unix.ts
    );
}
