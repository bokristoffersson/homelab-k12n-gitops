use chrono::{DateTime, Utc};
use rand::Rng;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

pub type TestDbPool = Pool<Postgres>;

/// Creates a test database connection pool
pub async fn create_test_pool(database_url: &str) -> Result<TestDbPool, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;

    Ok(pool)
}

/// Sets up the test database schema
pub async fn setup_test_schema(pool: &TestDbPool) -> Result<(), sqlx::Error> {
    // Create the heatpump table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS heatpump (
            ts TIMESTAMPTZ NOT NULL,
            device_id TEXT,
            room TEXT,
            outdoor_temp DOUBLE PRECISION,
            supplyline_temp DOUBLE PRECISION,
            returnline_temp DOUBLE PRECISION,
            hotwater_temp BIGINT,
            brine_out_temp BIGINT,
            brine_in_temp BIGINT,
            integral BIGINT,
            flowlinepump_speed BIGINT,
            brinepump_speed BIGINT,
            runtime_compressor BIGINT,
            runtime_hotwater BIGINT,
            runtime_3kw BIGINT,
            runtime_6kw BIGINT,
            brinepump_on BOOLEAN,
            compressor_on BOOLEAN,
            flowlinepump_on BOOLEAN,
            hotwater_production BOOLEAN,
            circulation_pump BOOLEAN,
            aux_heater_3kw_on BOOLEAN,
            aux_heater_6kw_on BOOLEAN
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Try to create hypertable if TimescaleDB is available
    let _ = sqlx::query("SELECT create_hypertable('heatpump', 'ts', if_not_exists => TRUE)")
        .execute(pool)
        .await;

    Ok(())
}

/// Cleans up test data
pub async fn cleanup_test_data(pool: &TestDbPool) -> Result<(), sqlx::Error> {
    sqlx::query("TRUNCATE TABLE heatpump")
        .execute(pool)
        .await?;
    Ok(())
}


/// Inserts a test reading into the database
pub async fn insert_test_reading(
    pool: &TestDbPool,
    device_id: Option<String>,
    room: Option<String>,
    ts: Option<DateTime<Utc>>,
) -> Result<(), sqlx::Error> {
    let mut rng = rand::thread_rng();
    let timestamp = ts.unwrap_or_else(|| Utc::now() - chrono::Duration::seconds(rng.gen_range(0..86400)));

    let device_id_val = device_id.unwrap_or_else(|| format!("device-{}", rng.gen_range(1..10)));
    let room_val = room.unwrap_or_else(|| format!("room-{}", rng.gen_range(1..5)));

    sqlx::query(
        r#"
        INSERT INTO heatpump (
            ts, device_id, room, outdoor_temp, supplyline_temp, returnline_temp,
            hotwater_temp, brine_out_temp, brine_in_temp, integral,
            flowlinepump_speed, brinepump_speed, runtime_compressor, runtime_hotwater,
            runtime_3kw, runtime_6kw, brinepump_on, compressor_on, flowlinepump_on,
            hotwater_production, circulation_pump, aux_heater_3kw_on, aux_heater_6kw_on
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23
        )
        "#,
    )
    .bind(timestamp)
    .bind(&device_id_val)
    .bind(&room_val)
    .bind(rng.gen_range(-10.0..30.0) as f64)
    .bind(rng.gen_range(30.0..60.0) as f64)
    .bind(rng.gen_range(25.0..55.0) as f64)
    .bind(rng.gen_range(40..70) as i64)
    .bind(rng.gen_range(0..20) as i64)
    .bind(rng.gen_range(0..20) as i64)
    .bind(rng.gen_range(0..1000) as i64)
    .bind(rng.gen_range(0..100) as i64)
    .bind(rng.gen_range(0..100) as i64)
    .bind(rng.gen_range(0..10000) as i64)
    .bind(rng.gen_range(0..10000) as i64)
    .bind(rng.gen_range(0..10000) as i64)
    .bind(rng.gen_range(0..10000) as i64)
    .bind(rng.gen_bool(0.5))
    .bind(rng.gen_bool(0.5))
    .bind(rng.gen_bool(0.5))
    .bind(rng.gen_bool(0.3))
    .bind(rng.gen_bool(0.5))
    .bind(rng.gen_bool(0.2))
    .bind(rng.gen_bool(0.1))
    .execute(pool)
    .await?;

    Ok(())
}

/// Inserts multiple test readings
pub async fn insert_test_readings(
    pool: &TestDbPool,
    count: usize,
    device_id: Option<String>,
    room: Option<String>,
) -> Result<(), sqlx::Error> {
    for i in 0..count {
        let ts = Utc::now() - chrono::Duration::hours(count as i64 - i as i64);
        insert_test_reading(pool, device_id.clone(), room.clone(), Some(ts)).await?;
    }
    Ok(())
}

