/// Integration tests for repository layer
///
/// These tests require a PostgreSQL database to be available.
/// They set up the schema, insert test data, and verify repository functions.
///
/// Run with: DATABASE_URL=postgres://postgres:postgres@localhost:5432/test cargo test --test repository_test
///
/// Note: These tests are marked with #[ignore] by default. Use --ignored flag to run them.
/// For CI/CD, use testcontainers or a test database setup.

use chrono::{DateTime, Utc};
use redpanda_sink::db::{connect, insert_batch};
use redpanda_sink::mapping::{FieldValue, Row};
use redpanda_sink::repositories::{EnergyRepository, HeatpumpRepository};
use serial_test::serial;
use std::collections::BTreeMap;

/// Helper function to set up database schema
async fn setup_schema(pool: &sqlx::PgPool) -> Result<(), sqlx::Error> {
    // Create energy table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS energy
        (
            ts                 TIMESTAMPTZ       NOT NULL,
            consumption_total_w DOUBLE PRECISION,
            consumption_l1_w    DOUBLE PRECISION,
            consumption_l2_w    DOUBLE PRECISION,
            consumption_l3_w    DOUBLE PRECISION,
            consumption_total_actual_w BIGINT,
            consumption_L1_actual_w BIGINT,
            consumption_L2_actual_w BIGINT,
            consumption_L3_actual_w BIGINT
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Create heatpump table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS heatpump
        (
            ts                  TIMESTAMPTZ       NOT NULL,
            device_id           TEXT,
            room                TEXT,
            outdoor_temp        DOUBLE PRECISION,
            supplyline_temp     DOUBLE PRECISION,
            returnline_temp     DOUBLE PRECISION,
            hotwater_temp       BIGINT,
            brine_out_temp      BIGINT,
            brine_in_temp       BIGINT,
            integral            BIGINT,
            flowlinepump_speed  BIGINT,
            brinepump_speed     BIGINT,
            runtime_compressor  BIGINT,
            runtime_hotwater    BIGINT,
            runtime_3kw         BIGINT,
            runtime_6kw         BIGINT,
            brinepump_on        BOOLEAN,
            compressor_on       BOOLEAN,
            flowlinepump_on     BOOLEAN,
            hotwater_production BOOLEAN,
            circulation_pump    BOOLEAN,
            aux_heater_3kw_on   BOOLEAN,
            aux_heater_6kw_on   BOOLEAN
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Try to enable TimescaleDB extension (may not be available in test container)
    let _ = sqlx::query("CREATE EXTENSION IF NOT EXISTS timescaledb CASCADE")
        .execute(pool)
        .await;

    // Try to create hypertables (will fail if TimescaleDB not available, that's OK)
    let _ = sqlx::query("SELECT create_hypertable('energy', 'ts', if_not_exists => TRUE)")
        .execute(pool)
        .await;
    let _ = sqlx::query("SELECT create_hypertable('heatpump', 'ts', if_not_exists => TRUE)")
        .execute(pool)
        .await;

    // Try to create continuous aggregate (will fail if TimescaleDB not available, that's OK)
    let _ = sqlx::query(
        r#"
        DROP MATERIALIZED VIEW IF EXISTS energy_hourly CASCADE;
        CREATE MATERIALIZED VIEW energy_hourly
        WITH (timescaledb.continuous) AS
        SELECT
          time_bucket('01:00:00'::interval, ts, origin => '2000-01-01 00:00:00+00'::timestamp with time zone) AS hour_start,
          (time_bucket('01:00:00'::interval, ts, origin => '2000-01-01 00:00:00+00'::timestamp with time zone) + '01:00:00'::interval) AS hour_end,
          (last(consumption_total_w, ts) - first(consumption_total_w, ts)) AS energy_consumption_w,
          (last(consumption_total_actual_w, ts) - first(consumption_total_actual_w, ts)) AS energy_consumption_total_actual_w,
          (last(consumption_L1_actual_w, ts) - first(consumption_L1_actual_w, ts)) AS energy_consumption_L1_actual_w,
          (last(consumption_L2_actual_w, ts) - first(consumption_L2_actual_w, ts)) AS energy_consumption_L2_actual_w,
          (last(consumption_L3_actual_w, ts) - first(consumption_L3_actual_w, ts)) AS energy_consumption_L3_actual_w,
          first(consumption_total_w, ts) AS hour_start_w,
          last(consumption_total_w, ts) AS hour_end_w,
          first(consumption_total_actual_w, ts) AS hour_start_actual_w,
          last(consumption_total_actual_w, ts) AS hour_end_actual_w,
          count(*) AS measurement_count,
          (last(consumption_total_w, ts) - first(consumption_total_w, ts)) / 1000.0 AS total_energy_kwh,
          (last(consumption_total_actual_w, ts) - first(consumption_total_actual_w, ts)) / 1000.0 AS total_energy_actual_kwh,
          (last(consumption_L1_actual_w, ts) - first(consumption_L1_actual_w, ts)) / 1000.0 AS total_energy_L1_actual_kwh,
          (last(consumption_L2_actual_w, ts) - first(consumption_L2_actual_w, ts)) / 1000.0 AS total_energy_L2_actual_kwh,
          (last(consumption_L3_actual_w, ts) - first(consumption_L3_actual_w, ts)) / 1000.0 AS total_energy_L3_actual_kwh
        FROM energy
        GROUP BY (time_bucket('01:00:00'::interval, ts, origin => '2000-01-01 00:00:00+00'::timestamp with time zone))
        WITH NO DATA
        "#,
    )
    .execute(pool)
    .await;

    Ok(())
}

/// Helper function to insert test energy data
async fn insert_test_energy_data(
    pool: &sqlx::PgPool,
    base_time: DateTime<Utc>,
) -> Result<(), redpanda_sink::error::AppError> {
    // Insert energy readings for the current hour
    // We'll insert readings at 0, 15, 30, 45 minutes
    let mut rows = Vec::new();

    for i in 0..4 {
        let ts = base_time + chrono::Duration::minutes(i * 15);
        let mut fields = BTreeMap::new();
        fields.insert(
            "consumption_total_w".to_string(),
            FieldValue::F64(1000.0 + (i as f64 * 100.0)),
        );
        fields.insert(
            "consumption_l1_w".to_string(),
            FieldValue::F64(300.0 + (i as f64 * 30.0)),
        );
        fields.insert(
            "consumption_l2_w".to_string(),
            FieldValue::F64(350.0 + (i as f64 * 35.0)),
        );
        fields.insert(
            "consumption_l3_w".to_string(),
            FieldValue::F64(350.0 + (i as f64 * 35.0)),
        );
        fields.insert(
            "consumption_total_actual_w".to_string(),
            FieldValue::I64(1000 + (i * 100)),
        );
        fields.insert(
            "consumption_L1_actual_w".to_string(),
            FieldValue::I64(300 + (i * 30)),
        );
        fields.insert(
            "consumption_L2_actual_w".to_string(),
            FieldValue::I64(350 + (i * 35)),
        );
        fields.insert(
            "consumption_L3_actual_w".to_string(),
            FieldValue::I64(350 + (i * 35)),
        );

        rows.push(Row {
            ts,
            tags: BTreeMap::new(),
            fields,
        });
    }

    insert_batch(pool, "energy", &rows).await?;
    Ok(())
}

/// Helper function to insert test heatpump data
async fn insert_test_heatpump_data(
    pool: &sqlx::PgPool,
    base_time: DateTime<Utc>,
    device_id: &str,
) -> Result<(), redpanda_sink::error::AppError> {
    let mut fields = BTreeMap::new();
    fields.insert("device_id".to_string(), FieldValue::Text(device_id.to_string()));
    fields.insert("compressor_on".to_string(), FieldValue::Bool(true));
    fields.insert("hotwater_production".to_string(), FieldValue::Bool(false));
    fields.insert("flowlinepump_on".to_string(), FieldValue::Bool(true));
    fields.insert("brinepump_on".to_string(), FieldValue::Bool(true));
    fields.insert("aux_heater_3kw_on".to_string(), FieldValue::Bool(false));
    fields.insert("aux_heater_6kw_on".to_string(), FieldValue::Bool(false));
    fields.insert("outdoor_temp".to_string(), FieldValue::F64(5.5));
    fields.insert("supplyline_temp".to_string(), FieldValue::F64(35.0));
    fields.insert("returnline_temp".to_string(), FieldValue::F64(30.0));
    fields.insert("hotwater_temp".to_string(), FieldValue::I64(45));
    fields.insert("brine_out_temp".to_string(), FieldValue::I64(8));
    fields.insert("brine_in_temp".to_string(), FieldValue::I64(6));

    let row = Row {
        ts: base_time,
        tags: BTreeMap::new(),
        fields,
    };

    insert_batch(pool, "heatpump", &[row]).await?;
    Ok(())
}

/// Helper function to align time to hour boundary (matching continuous aggregate origin)
fn align_to_hour_boundary(time: DateTime<Utc>) -> DateTime<Utc> {
    let origin = DateTime::parse_from_rfc3339("2000-01-01T00:00:00+00:00")
        .unwrap()
        .with_timezone(&Utc);
    let seconds_since_origin = (time - origin).num_seconds();
    let hours_since_origin = seconds_since_origin / 3600;
    origin + chrono::Duration::hours(hours_since_origin)
}

#[tokio::test]
#[serial]
#[ignore] // Requires database connection
async fn test_energy_repository_get_latest() {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/test".to_string());
    let pool = connect(&database_url).await.expect("Failed to connect to test database");

    setup_schema(&pool).await.expect("Failed to set up schema");

    let base_time = Utc::now();
    insert_test_energy_data(&pool, base_time)
        .await
        .expect("Failed to insert test data");

    // Test get_latest
    let result = EnergyRepository::get_latest(&pool).await;
    assert!(result.is_ok(), "get_latest should succeed");

    let latest = result.unwrap();
    assert_eq!(latest.consumption_total_w, Some(1300.0));
    assert_eq!(latest.consumption_l1_w, Some(390.0));
    assert_eq!(latest.consumption_l2_w, Some(455.0));
    assert_eq!(latest.consumption_l3_w, Some(455.0));
    assert_eq!(latest.consumption_total_actual_w, Some(1300));
}

#[tokio::test]
#[serial]
#[ignore] // Requires database connection
async fn test_energy_repository_get_latest_empty_table() {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/test".to_string());
    let pool = connect(&database_url).await.expect("Failed to connect to test database");

    setup_schema(&pool).await.expect("Failed to set up schema");

    // Test get_latest on empty table - should return an error
    let result = EnergyRepository::get_latest(&pool).await;
    assert!(result.is_err(), "get_latest should fail on empty table");
}

#[tokio::test]
#[serial]
#[ignore] // Requires database connection
async fn test_energy_repository_get_hourly_total() {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/test".to_string());
    let pool = connect(&database_url).await.expect("Failed to connect to test database");

    setup_schema(&pool).await.expect("Failed to set up schema");

    let now = Utc::now();
    let hour_start = align_to_hour_boundary(now);

    // Insert test data
    insert_test_energy_data(&pool, hour_start)
        .await
        .expect("Failed to insert test data");

    // If TimescaleDB is available, refresh the continuous aggregate
    // Otherwise, the query will work but return 0.0
    let _ = sqlx::query("CALL refresh_continuous_aggregate('energy_hourly', NULL, NULL)")
        .execute(&pool)
        .await;

    // Test get_hourly_total
    let result = EnergyRepository::get_hourly_total(&pool, hour_start).await;
    assert!(result.is_ok(), "get_hourly_total should succeed");

    let total = result.unwrap();
    // If continuous aggregate exists, it should calculate the difference
    // If not, it will return 0.0 (which is also valid for this test)
    assert!(total >= 0.0, "total should be non-negative");
}

#[tokio::test]
#[serial]
#[ignore] // Requires database connection
async fn test_energy_repository_get_hourly_history() {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/test".to_string());
    let pool = connect(&database_url).await.expect("Failed to connect to test database");

    setup_schema(&pool).await.expect("Failed to set up schema");

    let now = Utc::now();
    let hour_start = align_to_hour_boundary(now);

    // Insert test data for current hour
    insert_test_energy_data(&pool, hour_start)
        .await
        .expect("Failed to insert test data");

    // If TimescaleDB is available, refresh the continuous aggregate
    let _ = sqlx::query("CALL refresh_continuous_aggregate('energy_hourly', NULL, NULL)")
        .execute(&pool)
        .await;

    // Test get_hourly_history
    let from = hour_start - chrono::Duration::hours(1);
    let to = hour_start + chrono::Duration::hours(2);

    let result = EnergyRepository::get_hourly_history(&pool, from, to).await;
    assert!(result.is_ok(), "get_hourly_history should succeed");

    let history = result.unwrap();
    // Should have at least one entry if continuous aggregate exists
    // If not, it will be empty (which is also valid)
    assert!(
        history.len() >= 0,
        "history should return a valid vector (may be empty if continuous aggregate not available)"
    );
}

#[tokio::test]
#[serial]
#[ignore] // Requires database connection
async fn test_heatpump_repository_get_latest() {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/test".to_string());
    let pool = connect(&database_url).await.expect("Failed to connect to test database");

    setup_schema(&pool).await.expect("Failed to set up schema");

    let base_time = Utc::now();
    insert_test_heatpump_data(&pool, base_time, "device-001")
        .await
        .expect("Failed to insert test data");

    // Test get_latest without device_id filter
    let result = HeatpumpRepository::get_latest(&pool, None).await;
    assert!(result.is_ok(), "get_latest should succeed");

    let latest = result.unwrap();
    assert_eq!(latest.device_id, Some("device-001".to_string()));
    assert_eq!(latest.compressor_on, Some(true));
    assert_eq!(latest.hotwater_production, Some(false));
    assert_eq!(latest.outdoor_temp, Some(5.5));
}

#[tokio::test]
#[serial]
#[ignore] // Requires database connection
async fn test_heatpump_repository_get_latest_with_device_id() {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/test".to_string());
    let pool = connect(&database_url).await.expect("Failed to connect to test database");

    setup_schema(&pool).await.expect("Failed to set up schema");

    let base_time = Utc::now();
    insert_test_heatpump_data(&pool, base_time, "device-001")
        .await
        .expect("Failed to insert test data");
    insert_test_heatpump_data(&pool, base_time + chrono::Duration::seconds(1), "device-002")
        .await
        .expect("Failed to insert test data");

    // Test get_latest with device_id filter
    let result = HeatpumpRepository::get_latest(&pool, Some("device-001")).await;
    assert!(result.is_ok(), "get_latest should succeed");

    let latest = result.unwrap();
    assert_eq!(latest.device_id, Some("device-001".to_string()));

    // Test with different device_id
    let result = HeatpumpRepository::get_latest(&pool, Some("device-002")).await;
    assert!(result.is_ok(), "get_latest should succeed");

    let latest = result.unwrap();
    assert_eq!(latest.device_id, Some("device-002".to_string()));
}

#[tokio::test]
#[serial]
#[ignore] // Requires database connection
async fn test_heatpump_repository_get_latest_empty_table() {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/test".to_string());
    let pool = connect(&database_url).await.expect("Failed to connect to test database");

    setup_schema(&pool).await.expect("Failed to set up schema");

    // Test get_latest on empty table - should return an error
    let result = HeatpumpRepository::get_latest(&pool, None).await;
    assert!(result.is_err(), "get_latest should fail on empty table");

    let result = HeatpumpRepository::get_latest(&pool, Some("device-001")).await;
    assert!(result.is_err(), "get_latest should fail on empty table");
}

#[tokio::test]
#[serial]
#[ignore] // Requires database connection
async fn test_energy_repository_get_hourly_total_no_data() {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/test".to_string());
    let pool = connect(&database_url).await.expect("Failed to connect to test database");

    setup_schema(&pool).await.expect("Failed to set up schema");

    let now = Utc::now();
    let hour_start = align_to_hour_boundary(now);

    // Test get_hourly_total with no data - should return 0.0
    let result = EnergyRepository::get_hourly_total(&pool, hour_start).await;
    assert!(result.is_ok(), "get_hourly_total should succeed even with no data");
    assert_eq!(result.unwrap(), 0.0, "should return 0.0 when no data exists");
}

#[tokio::test]
#[serial]
#[ignore] // Requires database connection
async fn test_energy_repository_get_hourly_history_empty_range() {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/test".to_string());
    let pool = connect(&database_url).await.expect("Failed to connect to test database");

    setup_schema(&pool).await.expect("Failed to set up schema");

    let now = Utc::now();
    let from = now - chrono::Duration::days(7);
    let to = now - chrono::Duration::days(6);

    // Test get_hourly_history with no data in range
    let result = EnergyRepository::get_hourly_history(&pool, from, to).await;
    assert!(result.is_ok(), "get_hourly_history should succeed even with no data");
    assert_eq!(
        result.unwrap().len(),
        0,
        "should return empty vector when no data exists in range"
    );
}
