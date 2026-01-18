use crate::db::DbPool;
use crate::error::AppError;
use chrono::{DateTime, Utc};
use sqlx::{postgres::PgRow, FromRow, Row};

#[derive(Debug, Clone)]
pub struct TemperatureReading {
    pub time: DateTime<Utc>,
    pub device_id: Option<String>,
    pub mac_address: Option<String>,
    pub location: Option<String>,
    pub temperature_c: Option<f64>,
    pub temperature_f: Option<f64>,
    pub humidity: Option<f64>,
    pub wifi_rssi: Option<f64>,
    pub battery_voltage: Option<f64>,
    pub battery_percent: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct TemperatureLatest {
    pub time: DateTime<Utc>,
    pub location: Option<String>,
    pub temperature_c: Option<f64>,
    pub humidity: Option<f64>,
    pub battery_percent: Option<f64>,
}

pub struct TemperatureRepository;

impl<'r> FromRow<'r, PgRow> for TemperatureReading {
    fn from_row(row: &'r PgRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            time: row.try_get("time")?,
            device_id: row.try_get("device_id")?,
            mac_address: row.try_get("mac_address")?,
            location: row.try_get("location")?,
            temperature_c: row.try_get("temperature_c")?,
            temperature_f: row.try_get("temperature_f")?,
            humidity: row.try_get("humidity")?,
            wifi_rssi: row.try_get("wifi_rssi")?,
            battery_voltage: row.try_get("battery_voltage")?,
            battery_percent: row.try_get("battery_percent")?,
        })
    }
}

impl<'r> FromRow<'r, PgRow> for TemperatureLatest {
    fn from_row(row: &'r PgRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            time: row.try_get("time")?,
            location: row.try_get("location")?,
            temperature_c: row.try_get("temperature_c")?,
            humidity: row.try_get("humidity")?,
            battery_percent: row.try_get("battery_percent")?,
        })
    }
}

impl TemperatureRepository {
    pub async fn get_latest_by_location(
        pool: &DbPool,
        location: &str,
    ) -> Result<Option<TemperatureLatest>, AppError> {
        let result = sqlx::query_as::<_, TemperatureLatest>(
            r#"
            SELECT
                time,
                location,
                temperature_c,
                humidity,
                battery_percent
            FROM temperature_sensors
            WHERE location = $1
            ORDER BY time DESC
            LIMIT 1
            "#,
        )
        .bind(location)
        .fetch_optional(pool)
        .await?;

        Ok(result)
    }

    pub async fn get_history(
        pool: &DbPool,
        location: &str,
        hours: i32,
    ) -> Result<Vec<TemperatureReading>, AppError> {
        let results = sqlx::query_as::<_, TemperatureReading>(
            r#"
            SELECT
                time,
                device_id,
                mac_address,
                location,
                temperature_c,
                temperature_f,
                humidity,
                wifi_rssi,
                battery_voltage,
                battery_percent
            FROM temperature_sensors
            WHERE location = $1
                AND time > NOW() - INTERVAL '1 hour' * $2
            ORDER BY time ASC
            "#,
        )
        .bind(location)
        .bind(hours)
        .fetch_all(pool)
        .await?;

        Ok(results)
    }

    pub async fn get_all_latest(pool: &DbPool) -> Result<Vec<TemperatureLatest>, AppError> {
        let results = sqlx::query_as::<_, TemperatureLatest>(
            r#"
            SELECT DISTINCT ON (location)
                time,
                location,
                temperature_c,
                humidity,
                battery_percent
            FROM temperature_sensors
            WHERE location IS NOT NULL
            ORDER BY location, time DESC
            "#,
        )
        .fetch_all(pool)
        .await?;

        Ok(results)
    }
}
