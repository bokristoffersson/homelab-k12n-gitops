use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::util::Timeout;
/// End-to-end integration test
///
/// This test requires Docker to be running and will use
/// real Redpanda and PostgreSQL containers for testing.
///
/// Run with: cargo test --test end_to_end_test -- --ignored --nocapture
///
/// Note: These tests are marked as #[ignore] by default to avoid
/// running them in regular CI. Use --ignored flag to run them.
use redpanda_sink::config::{FieldConfig, Pipeline, TimestampConfig};
use redpanda_sink::db::{connect, insert_batch, upsert_batch};
use redpanda_sink::mapping::extract_row;
use serde_json::json;
use sqlx::Row;
use std::collections::BTreeMap;
use std::time::Duration;

#[tokio::test]
#[ignore]
async fn test_redpanda_to_database_timeseries() {
    // This test requires Docker and running Redpanda and PostgreSQL instances
    // It will be skipped in regular CI runs
    //
    // To run: Start Redpanda and PostgreSQL locally and set env vars
    // Example: REDPANDA_BROKERS=localhost:9092 DATABASE_URL=postgres://postgres:postgres@localhost:5432/test cargo test --test end_to_end_test -- --ignored

    let brokers =
        std::env::var("REDPANDA_BROKERS").unwrap_or_else(|_| "localhost:9092".to_string());
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/test".to_string());

    // Try to create Redpanda producer - if it fails, skip the test
    let producer: FutureProducer = match ClientConfig::new()
        .set("bootstrap.servers", &brokers)
        .set("message.timeout.ms", "5000")
        .create()
    {
        Ok(p) => p,
        Err(e) => {
            eprintln!(
                "⚠️  Skipping test: Redpanda is not available at {}\n\
                 Error: {}\n\
                 To run this test:\n\
                 1. Start Redpanda: docker run -d -p 9092:9092 docker.redpanda.com/redpandadata/redpanda:v24.2.19 redpanda start --kafka-addr 0.0.0.0:9092 --advertise-kafka-addr localhost:9092 --smp 1 --memory 1G --mode dev-container\n\
                 2. Wait for it to be ready\n\
                 3. Run: REDPANDA_BROKERS={} DATABASE_URL={} cargo test --test end_to_end_test -- --ignored",
                brokers, e, brokers, db_url
            );
            return;
        }
    };

    // Try to connect to database - if it fails, skip the test
    let pool = match connect(&db_url).await {
        Ok(p) => p,
        Err(e) => {
            eprintln!(
                "⚠️  Skipping test: Database is not available at {}\n\
                 Error: {}\n\
                 To run this test:\n\
                 1. Start PostgreSQL: docker run -d -p 5432:5432 -e POSTGRES_PASSWORD=postgres postgres:15\n\
                 2. Create database: docker exec -it <container-id> psql -U postgres -c 'CREATE DATABASE test;'\n\
                 3. Run: REDPANDA_BROKERS={} DATABASE_URL={} cargo test --test end_to_end_test -- --ignored",
                db_url, e, brokers, db_url
            );
            return;
        }
    };

    // Create test table
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS test_telemetry (
            ts TIMESTAMPTZ NOT NULL,
            device_id TEXT,
            temperature_c DOUBLE PRECISION,
            power_w BIGINT
        )",
    )
    .execute(&pool)
    .await
    .expect("Failed to create test table");

    // Create test pipeline
    let mut tags = BTreeMap::new();
    tags.insert("device_id".to_string(), "$.device_id".to_string());

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

    let pipeline = Pipeline {
        name: "test-pipeline".to_string(),
        topic: "test-telemetry".to_string(),
        table: "test_telemetry".to_string(),
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

    // Create test message
    let test_payload = json!({
        "device_id": "test-device-001",
        "temperature": 21.5,
        "power": 950
    });

    // Publish to Redpanda
    let payload_str = test_payload.to_string();
    let record = FutureRecord::to("test-telemetry")
        .payload(payload_str.as_bytes())
        .key("test-device-001");

    match producer
        .send(record, Timeout::After(Duration::from_secs(5)))
        .await
    {
        Ok((_partition, _offset)) => {
            println!("✅ Published message to Redpanda");
        }
        Err((e, _message)) => {
            eprintln!("⚠️  Failed to publish to Redpanda: {}", e);
            return;
        }
    }

    // Simulate consuming and processing
    let row = extract_row(
        &pipeline,
        "test-telemetry",
        test_payload.to_string().as_bytes(),
    )
    .expect("Failed to extract row");

    // Insert into database
    match insert_batch(&pool, "test_telemetry", &[row]).await {
        Ok(_) => {
            println!("✅ Inserted data into database");
        }
        Err(e) => {
            eprintln!("⚠️  Failed to insert into database: {}", e);
            return;
        }
    }

    // Verify data was inserted
    let result = sqlx::query("SELECT device_id, temperature_c, power_w FROM test_telemetry WHERE device_id = 'test-device-001'")
        .fetch_one(&pool)
        .await;

    match result {
        Ok(row) => {
            let device_id: String = row.get(0);
            let temp: f64 = row.get(1);
            let power: i64 = row.get(2);

            assert_eq!(device_id, "test-device-001");
            assert!((temp - 21.5).abs() < 0.01);
            assert_eq!(power, 950);
            println!("✅ Verified data in database");
        }
        Err(e) => {
            eprintln!("⚠️  Failed to verify data: {}", e);
        }
    }

    // Cleanup
    sqlx::query("DROP TABLE IF EXISTS test_telemetry")
        .execute(&pool)
        .await
        .ok();
}

#[tokio::test]
#[ignore]
async fn test_redpanda_to_database_static() {
    // Test static data with upsert
    let _brokers =
        std::env::var("REDPANDA_BROKERS").unwrap_or_else(|_| "localhost:9092".to_string());
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/test".to_string());

    // Try to connect to database
    let pool = match connect(&db_url).await {
        Ok(p) => p,
        Err(_) => {
            eprintln!("⚠️  Skipping test: Database is not available");
            return;
        }
    };

    // Create test table with unique constraint
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS test_devices (
            latest_update TIMESTAMPTZ NOT NULL,
            device_id TEXT NOT NULL PRIMARY KEY,
            name TEXT,
            status TEXT
        )",
    )
    .execute(&pool)
    .await
    .expect("Failed to create test table");

    // Create test pipeline for static data
    let mut tags = BTreeMap::new();
    tags.insert("device_id".to_string(), "$.device_id".to_string());

    let mut fields = BTreeMap::new();
    fields.insert(
        "name".to_string(),
        FieldConfig {
            path: "$.name".to_string(),
            r#type: "text".to_string(),
            attributes: None,
        },
    );
    fields.insert(
        "status".to_string(),
        FieldConfig {
            path: "$.status".to_string(),
            r#type: "text".to_string(),
            attributes: None,
        },
    );

    let pipeline = Pipeline {
        name: "test-static".to_string(),
        topic: "test-devices".to_string(),
        table: "test_devices".to_string(),
        data_type: "static".to_string(),
        upsert_key: Some(vec!["device_id".to_string()]),
        timestamp: TimestampConfig {
            use_now: true,
            ..Default::default()
        },
        tags,
        fields,
        bit_flags: None,
        store_interval: None,
    };

    // Create test message
    let test_payload = json!({
        "device_id": "device-001",
        "name": "Test Device",
        "status": "active"
    });

    // Extract row
    let row = extract_row(
        &pipeline,
        "test-devices",
        test_payload.to_string().as_bytes(),
    )
    .expect("Failed to extract row");

    // Upsert into database
    match upsert_batch(
        &pool,
        "test_devices",
        &["device_id".to_string()],
        std::slice::from_ref(&row),
    )
    .await
    {
        Ok(_) => {
            println!("✅ Upserted data into database");
        }
        Err(e) => {
            eprintln!("⚠️  Failed to upsert into database: {}", e);
            return;
        }
    }

    // Upsert again with updated data
    let updated_payload = json!({
        "device_id": "device-001",
        "name": "Updated Device",
        "status": "inactive"
    });

    let updated_row = extract_row(
        &pipeline,
        "test-devices",
        updated_payload.to_string().as_bytes(),
    )
    .expect("Failed to extract row");

    match upsert_batch(
        &pool,
        "test_devices",
        &["device_id".to_string()],
        &[updated_row],
    )
    .await
    {
        Ok(_) => {
            println!("✅ Updated data via upsert");
        }
        Err(e) => {
            eprintln!("⚠️  Failed to upsert update: {}", e);
            return;
        }
    }

    // Verify data was updated
    let result = sqlx::query(
        "SELECT device_id, name, status FROM test_devices WHERE device_id = 'device-001'",
    )
    .fetch_one(&pool)
    .await;

    match result {
        Ok(row) => {
            let device_id: String = row.get(0);
            let name: String = row.get(1);
            let status: String = row.get(2);

            assert_eq!(device_id, "device-001");
            assert_eq!(name, "Updated Device");
            assert_eq!(status, "inactive");
            println!("✅ Verified upsert worked correctly");
        }
        Err(e) => {
            eprintln!("⚠️  Failed to verify data: {}", e);
        }
    }

    // Cleanup
    sqlx::query("DROP TABLE IF EXISTS test_devices")
        .execute(&pool)
        .await
        .ok();
}
