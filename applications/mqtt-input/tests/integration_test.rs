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
        FieldValue::Bool(v) => assert!(*v),
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
        FieldValue::Bool(v) => assert!(*v),
        _ => panic!("Expected Bool"),
    }

    match row.fields.get("heating_mode").unwrap() {
        FieldValue::Bool(v) => assert!(!*v),
        _ => panic!("Expected Bool"),
    }

    match row.fields.get("hot_water_mode").unwrap() {
        FieldValue::Bool(v) => assert!(*v),
        _ => panic!("Expected Bool"),
    }

    match row.fields.get("circulation_pump").unwrap() {
        FieldValue::Bool(v) => assert!(*v),
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

/// Test multiple nested fields extraction (activeTotalConsumption and activeActualConsumption)
/// This test verifies that multiple nested fields can be extracted from the same message
#[tokio::test]
async fn test_multiple_nested_fields_extraction() {
    use mqtt_input::config::{FieldConfig, Pipeline};
    use mqtt_input::mapping::extract_row;
    use serde_json::json;

    // Configure activeTotalConsumption nested field
    let mut total_consumption_attrs = BTreeMap::new();
    total_consumption_attrs.insert("total".to_string(), "consumption_total_w".to_string());

    // Configure activeActualConsumption nested field
    let mut actual_consumption_attrs = BTreeMap::new();
    actual_consumption_attrs.insert(
        "total".to_string(),
        "consumption_total_actual_w".to_string(),
    );
    actual_consumption_attrs.insert("L1".to_string(), "consumption_L1_actual_w".to_string());
    actual_consumption_attrs.insert("L2".to_string(), "consumption_L2_actual_w".to_string());
    actual_consumption_attrs.insert("L3".to_string(), "consumption_L3_actual_w".to_string());

    let mut fields = BTreeMap::new();

    // Add activeTotalConsumption field
    fields.insert(
        "activeTotalConsumption".to_string(),
        FieldConfig {
            path: "$.activeTotalConsumption".to_string(),
            r#type: "nested".to_string(),
            attributes: Some(total_consumption_attrs),
        },
    );

    // Add activeActualConsumption field
    fields.insert(
        "activeActualConsumption".to_string(),
        FieldConfig {
            path: "$.activeActualConsumption".to_string(),
            r#type: "nested".to_string(),
            attributes: Some(actual_consumption_attrs),
        },
    );

    let pipeline = Pipeline {
        name: "energy".to_string(),
        topic: "saveeye/telemetry".to_string(),
        qos: 1,
        redpanda_topic: "energy-realtime".to_string(),
        timestamp: TimestampConfig {
            use_now: true,
            ..Default::default()
        },
        tags: BTreeMap::new(),
        fields,
        bit_flags: None,
        store_interval: None,
    };

    // Sample message matching the actual MQTT message structure
    let payload = json!({
        "saveeyeDeviceSerialNumber": "9MANZWNG",
        "meterType": "",
        "meterSerialNumber": "Not found",
        "timestamp": "2025-12-02T20:17:48",
        "wifiRssi": -89,
        "activeActualConsumption": {
            "total": 956,
            "L1": 638,
            "L2": 250,
            "L3": 67
        },
        "activeActualProduction": {
            "total": 0,
            "L1": 0,
            "L2": 0,
            "L3": 0
        },
        "activeTotalConsumption": {
            "total": 102795417
        },
        "activeTotalProduction": {
            "total": 83
        },
        "reactiveActualConsumption": {
            "total": 0
        },
        "reactiveActualProduction": {
            "total": 621
        },
        "reactiveTotalConsumption": {
            "total": 30100902
        },
        "reactiveTotalProduction": {
            "total": 15964771
        },
        "rmsVoltage": {
            "L1": 235,
            "L2": 236,
            "L3": 237
        },
        "rmsCurrent": {
            "L1": 3100,
            "L2": 1700,
            "L3": 400
        },
        "powerFactor": {
            "total": 100
        }
    });

    let row = extract_row(
        &pipeline,
        "saveeye/telemetry",
        payload.to_string().as_bytes(),
    )
    .unwrap();

    // Verify activeTotalConsumption extraction
    match row.fields.get("consumption_total_w") {
        Some(FieldValue::F64(v)) => assert_eq!(
            *v, 102795417.0,
            "activeTotalConsumption.total should be extracted"
        ),
        Some(other) => panic!("Expected F64 for consumption_total_w, got {:?}", other),
        None => {
            panic!("consumption_total_w field is missing - activeTotalConsumption not extracted")
        }
    }

    // Verify activeActualConsumption extraction - total
    match row.fields.get("consumption_total_actual_w") {
        Some(FieldValue::F64(v)) => assert_eq!(*v, 956.0, "activeActualConsumption.total should be extracted"),
        Some(other) => panic!("Expected F64 for consumption_total_actual_w, got {:?}", other),
        None => panic!("consumption_total_actual_w field is missing - activeActualConsumption.total not extracted"),
    }

    // Verify activeActualConsumption extraction - L1
    match row.fields.get("consumption_L1_actual_w") {
        Some(FieldValue::F64(v)) => {
            assert_eq!(*v, 638.0, "activeActualConsumption.L1 should be extracted")
        }
        Some(other) => panic!("Expected F64 for consumption_L1_actual_w, got {:?}", other),
        None => panic!(
            "consumption_L1_actual_w field is missing - activeActualConsumption.L1 not extracted"
        ),
    }

    // Verify activeActualConsumption extraction - L2
    match row.fields.get("consumption_L2_actual_w") {
        Some(FieldValue::F64(v)) => {
            assert_eq!(*v, 250.0, "activeActualConsumption.L2 should be extracted")
        }
        Some(other) => panic!("Expected F64 for consumption_L2_actual_w, got {:?}", other),
        None => panic!(
            "consumption_L2_actual_w field is missing - activeActualConsumption.L2 not extracted"
        ),
    }

    // Verify activeActualConsumption extraction - L3
    match row.fields.get("consumption_L3_actual_w") {
        Some(FieldValue::F64(v)) => {
            assert_eq!(*v, 67.0, "activeActualConsumption.L3 should be extracted")
        }
        Some(other) => panic!("Expected F64 for consumption_L3_actual_w, got {:?}", other),
        None => panic!(
            "consumption_L3_actual_w field is missing - activeActualConsumption.L3 not extracted"
        ),
    }

    // Verify all expected fields are present
    let expected_fields = vec![
        "consumption_total_w",
        "consumption_total_actual_w",
        "consumption_L1_actual_w",
        "consumption_L2_actual_w",
        "consumption_L3_actual_w",
    ];

    for field_name in &expected_fields {
        assert!(
            row.fields.contains_key(*field_name),
            "Field {} should be present in extracted fields. Available fields: {:?}",
            field_name,
            row.fields.keys().collect::<Vec<_>>()
        );
    }

    // Verify we have exactly the expected number of fields
    assert_eq!(
        row.fields.len(),
        expected_fields.len(),
        "Expected {} fields, but got {}. Fields: {:?}",
        expected_fields.len(),
        row.fields.len(),
        row.fields.keys().collect::<Vec<_>>()
    );
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

/// Test that one MQTT message can be processed by multiple pipelines
/// This verifies that when multiple pipelines match the same MQTT topic,
/// each pipeline processes the message and publishes to its respective Redpanda topic
#[tokio::test]
async fn test_multiple_pipelines_same_message() {
    use mqtt_input::config::{FieldConfig, Pipeline, TimestampConfig};
    use mqtt_input::ingest::Ingestor;
    use mqtt_input::redpanda::create_producer;
    use rdkafka::config::ClientConfig;
    use rdkafka::consumer::{Consumer, StreamConsumer};
    use rdkafka::Message;
    use serde_json::json;
    use std::time::Duration;
    use tokio::time::timeout;

    let brokers =
        std::env::var("REDPANDA_BROKERS").unwrap_or_else(|_| "localhost:9092".to_string());

    // Try to create Redpanda producer - if it fails, skip the test
    let producer = match create_producer(&brokers).await {
        Ok(p) => p,
        Err(e) => {
            eprintln!(
                "⚠️  Skipping test: Redpanda is not available at {}\n\
                 Error: {}\n\
                 To run this test, start Redpanda first.",
                brokers, e
            );
            return;
        }
    };
    let ingestor = Ingestor::new(producer);

    // Create first pipeline: extracts telemetry data
    let mut tags1 = BTreeMap::new();
    tags1.insert("device_id".to_string(), "$.Client_Name".to_string());

    let mut fields1 = BTreeMap::new();
    fields1.insert(
        "temperature".to_string(),
        FieldConfig {
            path: "$.d0".to_string(),
            r#type: "float".to_string(),
            attributes: None,
        },
    );
    fields1.insert(
        "supply_temp".to_string(),
        FieldConfig {
            path: "$.d5".to_string(),
            r#type: "float".to_string(),
            attributes: None,
        },
    );

    let pipeline1 = Pipeline {
        name: "heatpump-telemetry".to_string(),
        topic: "thermiq_heatpump/data".to_string(),
        qos: 1,
        redpanda_topic: "heatpump-telemetry".to_string(),
        timestamp: TimestampConfig {
            use_now: true,
            ..Default::default()
        },
        tags: tags1,
        fields: fields1,
        bit_flags: None,
        store_interval: None,
    };

    // Create second pipeline: extracts settings data from the same topic
    let mut tags2 = BTreeMap::new();
    tags2.insert("device_id".to_string(), "$.Client_Name".to_string());

    let mut fields2 = BTreeMap::new();
    fields2.insert(
        "d50".to_string(),
        FieldConfig {
            path: "$.d50".to_string(),
            r#type: "float".to_string(),
            attributes: None,
        },
    );
    fields2.insert(
        "d51".to_string(),
        FieldConfig {
            path: "$.d51".to_string(),
            r#type: "int".to_string(),
            attributes: None,
        },
    );
    fields2.insert(
        "d52".to_string(),
        FieldConfig {
            path: "$.d52".to_string(),
            r#type: "int".to_string(),
            attributes: None,
        },
    );

    let pipeline2 = Pipeline {
        name: "heatpump-settings".to_string(),
        topic: "thermiq_heatpump/data".to_string(), // Same topic as pipeline1
        qos: 1,
        redpanda_topic: "heatpump-settings".to_string(), // Different Redpanda topic
        timestamp: TimestampConfig {
            use_now: true,
            ..Default::default()
        },
        tags: tags2,
        fields: fields2,
        bit_flags: None,
        store_interval: None,
    };

    // Create third pipeline: uses wildcard to match the same topic
    let mut tags3 = BTreeMap::new();
    tags3.insert("device_id".to_string(), "$.Client_Name".to_string());

    let mut fields3 = BTreeMap::new();
    fields3.insert(
        "all_data".to_string(),
        FieldConfig {
            path: "$".to_string(), // Extract entire message as text
            r#type: "text".to_string(),
            attributes: None,
        },
    );

    let pipeline3 = Pipeline {
        name: "heatpump-raw".to_string(),
        topic: "thermiq_heatpump/#".to_string(), // Wildcard matches the topic
        qos: 1,
        redpanda_topic: "heatpump-raw".to_string(),
        timestamp: TimestampConfig {
            use_now: true,
            ..Default::default()
        },
        tags: tags3,
        fields: fields3,
        bit_flags: None,
        store_interval: None,
    };

    // All three pipelines should process the same message
    let pipelines = vec![pipeline1, pipeline2, pipeline3];

    // Create a message that contains both telemetry and settings data
    let test_payload = json!({
        "Client_Name": "test-heatpump-01",
        "d0": 4.5,      // outdoor_temp (for pipeline1)
        "d5": 39.0,     // supplyline_temp (for pipeline1)
        "d50": 22.5,    // indoor_target_temp (for pipeline2)
        "d51": 1,       // mode (for pipeline2)
        "d52": 2        // curve (for pipeline2)
    });

    // Process the message - should be handled by all 3 pipelines
    let result = ingestor
        .handle_message(
            &pipelines,
            "thermiq_heatpump/data",
            test_payload.to_string().as_bytes(),
        )
        .await;

    if let Err(e) = &result {
        eprintln!(
            "⚠️  Skipping test: Failed to publish to Redpanda at {}\n\
             Error: {}\n\
             Redpanda is not running. Start Redpanda to run this test.",
            brokers, e
        );
        return;
    }

    assert!(
        result.is_ok(),
        "Message processing should succeed for all pipelines"
    );

    // Verify messages were published to all three Redpanda topics
    let topics = vec!["heatpump-telemetry", "heatpump-settings", "heatpump-raw"];
    let mut consumed_messages = Vec::new();

    for topic in &topics {
        let consumer: StreamConsumer = ClientConfig::new()
            .set("bootstrap.servers", &brokers)
            .set("group.id", &format!("test-consumer-{}", topic))
            .set("auto.offset.reset", "earliest")
            .create()
            .expect("Failed to create consumer");

        consumer.subscribe(&[topic]).expect("Failed to subscribe");

        // Consume message with timeout
        let message_result = timeout(Duration::from_secs(10), consumer.recv()).await;

        if let Ok(Ok(message)) = message_result {
            let payload = message.payload().expect("Message should have payload");
            let json: serde_json::Value =
                serde_json::from_slice(payload).expect("Payload should be valid JSON");

            consumed_messages.push((topic.to_string(), json));
        } else {
            panic!("Failed to consume message from topic: {}", topic);
        }
    }

    // Verify all three topics received messages
    assert_eq!(
        consumed_messages.len(),
        3,
        "Expected messages in all 3 topics, got {}",
        consumed_messages.len()
    );

    // Verify heatpump-telemetry topic
    let telemetry_msg = consumed_messages
        .iter()
        .find(|(t, _)| t == "heatpump-telemetry")
        .expect("heatpump-telemetry message not found");
    let telemetry_tags = telemetry_msg.1.get("tags").unwrap().as_object().unwrap();
    let telemetry_fields = telemetry_msg.1.get("fields").unwrap().as_object().unwrap();
    assert_eq!(
        telemetry_tags.get("device_id").unwrap().as_str().unwrap(),
        "test-heatpump-01"
    );
    assert_eq!(
        telemetry_fields
            .get("temperature")
            .unwrap()
            .as_f64()
            .unwrap(),
        4.5
    );
    assert_eq!(
        telemetry_fields
            .get("supply_temp")
            .unwrap()
            .as_f64()
            .unwrap(),
        39.0
    );

    // Verify heatpump-settings topic
    let settings_msg = consumed_messages
        .iter()
        .find(|(t, _)| t == "heatpump-settings")
        .expect("heatpump-settings message not found");
    let settings_tags = settings_msg.1.get("tags").unwrap().as_object().unwrap();
    let settings_fields = settings_msg.1.get("fields").unwrap().as_object().unwrap();
    assert_eq!(
        settings_tags.get("device_id").unwrap().as_str().unwrap(),
        "test-heatpump-01"
    );
    assert_eq!(settings_fields.get("d50").unwrap().as_f64().unwrap(), 22.5);
    assert_eq!(settings_fields.get("d51").unwrap().as_i64().unwrap(), 1);
    assert_eq!(settings_fields.get("d52").unwrap().as_i64().unwrap(), 2);

    // Verify heatpump-raw topic
    let raw_msg = consumed_messages
        .iter()
        .find(|(t, _)| t == "heatpump-raw")
        .expect("heatpump-raw message not found");
    let raw_tags = raw_msg.1.get("tags").unwrap().as_object().unwrap();
    let raw_fields = raw_msg.1.get("fields").unwrap().as_object().unwrap();
    assert_eq!(
        raw_tags.get("device_id").unwrap().as_str().unwrap(),
        "test-heatpump-01"
    );
    // The all_data field should contain the JSON string representation
    assert!(raw_fields.contains_key("all_data"));

    println!("✓ Multiple pipelines test passed: One message processed by 3 pipelines and published to 3 Redpanda topics");
}
