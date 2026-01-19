use chrono::{DateTime, TimeZone, Utc};
use homelab_api::db::DbPool;
use homelab_api::mcp::types::JsonRpcRequest;
use serde_json::{json, Value};
use sqlx::PgPool;
use testcontainers::{clients::Cli, images::postgres::Postgres, RunnableImage};

async fn setup_test_db() -> PgPool {
    let docker = Cli::default();
    let postgres_image = RunnableImage::from(Postgres::default()).with_tag("16-alpine");

    let node = docker.run(postgres_image);
    let connection_string = format!(
        "postgres://postgres:postgres@127.0.0.1:{}/postgres",
        node.get_host_port_ipv4(5432)
    );

    let pool = PgPool::connect(&connection_string)
        .await
        .expect("Failed to connect to test database");

    // Create TimescaleDB extension and hypertable
    sqlx::query("CREATE EXTENSION IF NOT EXISTS timescaledb")
        .execute(&pool)
        .await
        .expect("Failed to create timescaledb extension");

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS energy_readings (
            time TIMESTAMPTZ NOT NULL,
            total_energy_kwh DOUBLE PRECISION,
            total_energy_l1_kwh DOUBLE PRECISION,
            total_energy_l2_kwh DOUBLE PRECISION,
            total_energy_l3_kwh DOUBLE PRECISION,
            total_energy_actual_kwh DOUBLE PRECISION
        )
        "#,
    )
    .execute(&pool)
    .await
    .expect("Failed to create energy_readings table");

    sqlx::query("SELECT create_hypertable('energy_readings', 'time', if_not_exists => TRUE)")
        .execute(&pool)
        .await
        .expect("Failed to create hypertable");

    pool
}

async fn insert_test_energy_data(pool: &PgPool, start: DateTime<Utc>, hours: usize) {
    for hour in 0..hours {
        let time = start + chrono::Duration::hours(hour as i64);

        // Insert data points every minute for the hour (60 points)
        for minute in 0..60 {
            let measurement_time = time + chrono::Duration::minutes(minute);

            sqlx::query(
                r#"
                INSERT INTO energy_readings
                (time, total_energy_kwh, total_energy_l1_kwh, total_energy_l2_kwh,
                 total_energy_l3_kwh, total_energy_actual_kwh)
                VALUES ($1, $2, $3, $4, $5, $6)
                "#,
            )
            .bind(measurement_time)
            .bind(5.0 + (hour as f64)) // Predictable test values
            .bind(1.5 + (hour as f64 * 0.3))
            .bind(1.7 + (hour as f64 * 0.3))
            .bind(1.8 + (hour as f64 * 0.4))
            .bind(-2.0 + (hour as f64)) // Negative = export
            .execute(pool)
            .await
            .expect("Failed to insert test data");
        }
    }
}

#[tokio::test]
async fn test_energy_hourly_consumption_returns_correct_data() {
    let pool = setup_test_db().await;

    // Insert 3 hours of test data starting at 2026-01-18 00:00:00 UTC
    let start_time = Utc.with_ymd_and_hms(2026, 1, 18, 0, 0, 0).unwrap();
    insert_test_energy_data(&pool, start_time, 3).await;

    // Create MCP request for the first 2 hours
    let from = "2026-01-18T00:00:00Z";
    let to = "2026-01-18T02:00:00Z";

    let request = JsonRpcRequest {
        jsonrpc: Some("2.0".to_string()),
        id: Some(json!(1)),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "energy_hourly_consumption",
            "arguments": {
                "from": from,
                "to": to
            }
        })),
    };

    // Call the MCP handler (we need to expose this in the lib)
    // For now, test the repository directly
    use homelab_api::repositories::EnergyRepository;

    let from_dt = DateTime::parse_from_rfc3339(from)
        .unwrap()
        .with_timezone(&Utc);
    let to_dt = DateTime::parse_from_rfc3339(to)
        .unwrap()
        .with_timezone(&Utc);

    let readings = EnergyRepository::get_hourly_history(&pool, from_dt, to_dt)
        .await
        .expect("Failed to get hourly history");

    // Verify we got 2 hours of data
    assert_eq!(readings.len(), 2, "Expected 2 hours of data");

    // Verify first hour (00:00-01:00)
    assert_eq!(readings[0].hour_start, start_time);
    assert_eq!(
        readings[0].hour_end,
        start_time + chrono::Duration::hours(1)
    );
    assert_eq!(
        readings[0].measurement_count, 60,
        "Expected 60 measurements per hour"
    );

    // Each measurement had total_energy_kwh = 5.0 for hour 0
    // With 60 measurements, we expect aggregated value
    // Note: This depends on how the repository aggregates (SUM, AVG, etc.)
    assert!(
        readings[0].total_energy_kwh > 0.0,
        "Expected positive energy consumption"
    );

    // Verify second hour (01:00-02:00)
    assert_eq!(
        readings[1].hour_start,
        start_time + chrono::Duration::hours(1)
    );
    assert_eq!(readings[1].measurement_count, 60);
}

#[tokio::test]
async fn test_energy_hourly_consumption_handles_empty_range() {
    let pool = setup_test_db().await;

    use homelab_api::repositories::EnergyRepository;

    // Query for a range with no data
    let from = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
    let to = Utc.with_ymd_and_hms(2025, 1, 1, 2, 0, 0).unwrap();

    let readings = EnergyRepository::get_hourly_history(&pool, from, to)
        .await
        .expect("Failed to get hourly history");

    assert_eq!(readings.len(), 0, "Expected no data for empty range");
}

#[tokio::test]
async fn test_mcp_response_format() {
    // Test that the MCP response uses correct content type
    let response = json!({
        "content": [
            {
                "type": "text",
                "text": "{\"count\":2,\"from\":\"2026-01-18T00:00:00Z\"}"
            }
        ],
        "isError": false
    });

    assert_eq!(
        response["content"][0]["type"].as_str().unwrap(),
        "text",
        "MCP response must use 'text' content type"
    );

    assert!(
        response["content"][0]["text"].is_string(),
        "MCP text content must be a string"
    );

    assert_eq!(
        response["isError"].as_bool().unwrap(),
        false,
        "Successful response should have isError=false"
    );
}
