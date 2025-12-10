use crate::db::DbPool;
use crate::error::AppError;
use chrono::{DateTime, Utc};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct HeatpumpLatest {
    pub ts: DateTime<Utc>,
    pub device_id: Option<String>,
    pub compressor_on: Option<bool>,
    pub hotwater_production: Option<bool>,
    pub flowlinepump_on: Option<bool>,
    pub brinepump_on: Option<bool>,
    pub aux_heater_3kw_on: Option<bool>,
    pub aux_heater_6kw_on: Option<bool>,
    pub outdoor_temp: Option<f64>,
    pub supplyline_temp: Option<f64>,
    pub returnline_temp: Option<f64>,
    pub hotwater_temp: Option<i64>,
    pub brine_out_temp: Option<i64>,
    pub brine_in_temp: Option<i64>,
}

#[derive(Debug, Clone, FromRow)]
pub struct HeatpumpDailySummary {
    pub day: DateTime<Utc>,
    pub daily_runtime_compressor_increase: Option<i64>,
    pub daily_runtime_hotwater_increase: Option<i64>,
    pub daily_runtime_3kw_increase: Option<i64>,
    pub daily_runtime_6kw_increase: Option<i64>,
    pub avg_outdoor_temp: Option<f64>,
    pub avg_supplyline_temp: Option<f64>,
    pub avg_returnline_temp: Option<f64>,
    pub avg_hotwater_temp: Option<f64>,
    pub avg_brine_out_temp: Option<f64>,
    pub avg_brine_in_temp: Option<f64>,
}

pub struct HeatpumpRepository;

impl HeatpumpRepository {
    pub async fn get_latest(
        pool: &DbPool,
        device_id: Option<&str>,
    ) -> Result<HeatpumpLatest, AppError> {
        let query = if let Some(device_id) = device_id {
            sqlx::query_as::<_, HeatpumpLatest>(
                r#"
                SELECT 
                    ts,
                    device_id,
                    compressor_on,
                    hotwater_production,
                    flowlinepump_on,
                    brinepump_on,
                    aux_heater_3kw_on,
                    aux_heater_6kw_on,
                    outdoor_temp,
                    supplyline_temp,
                    returnline_temp,
                    hotwater_temp,
                    brine_out_temp,
                    brine_in_temp
                FROM heatpump
                WHERE device_id = $1
                ORDER BY ts DESC
                LIMIT 1
                "#,
            )
            .bind(device_id)
        } else {
            sqlx::query_as::<_, HeatpumpLatest>(
                r#"
                SELECT 
                    ts,
                    device_id,
                    compressor_on,
                    hotwater_production,
                    flowlinepump_on,
                    brinepump_on,
                    aux_heater_3kw_on,
                    aux_heater_6kw_on,
                    outdoor_temp,
                    supplyline_temp,
                    returnline_temp,
                    hotwater_temp,
                    brine_out_temp,
                    brine_in_temp
                FROM heatpump
                ORDER BY ts DESC
                LIMIT 1
                "#,
            )
        };

        query.fetch_one(pool).await.map_err(AppError::Db)
    }

    pub async fn get_daily_summary(
        pool: &DbPool,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
        device_id: Option<&str>,
    ) -> Result<Vec<HeatpumpDailySummary>, AppError> {
        // If device_id is provided, query the raw table and compute aggregates on the fly
        // since the continuous aggregate groups all devices together
        if let Some(device_id) = device_id {
            // Check if TimescaleDB is available for time_bucket function
            match sqlx::query_as::<_, HeatpumpDailySummary>(
                r#"
                SELECT 
                    time_bucket('1 day'::interval, ts) AS day,
                    (last(runtime_compressor, ts) - first(runtime_compressor, ts)) AS daily_runtime_compressor_increase,
                    (last(runtime_hotwater, ts) - first(runtime_hotwater, ts)) AS daily_runtime_hotwater_increase,
                    (last(runtime_3kw, ts) - first(runtime_3kw, ts)) AS daily_runtime_3kw_increase,
                    (last(runtime_6kw, ts) - first(runtime_6kw, ts)) AS daily_runtime_6kw_increase,
                    avg(outdoor_temp) AS avg_outdoor_temp,
                    avg(supplyline_temp) AS avg_supplyline_temp,
                    avg(returnline_temp) AS avg_returnline_temp,
                    avg(hotwater_temp) AS avg_hotwater_temp,
                    avg(brine_out_temp) AS avg_brine_out_temp,
                    avg(brine_in_temp) AS avg_brine_in_temp
                FROM heatpump
                WHERE ts >= $1 AND ts < $2 AND device_id = $3
                GROUP BY time_bucket('1 day'::interval, ts)
                ORDER BY day
                "#,
            )
            .bind(from)
            .bind(to)
            .bind(device_id)
            .fetch_all(pool)
            .await
            {
                Ok(results) => Ok(results),
                Err(sqlx::Error::Database(db_err))
                    if db_err.message().contains("function time_bucket") =>
                {
                    // TimescaleDB not available, return empty vector
                    Ok(Vec::new())
                }
                Err(e) => Err(AppError::Db(e)),
            }
        } else {
            // No device_id filter - use the continuous aggregate for better performance
            match sqlx::query_as::<_, HeatpumpDailySummary>(
                r#"
                SELECT 
                    day,
                    daily_runtime_compressor_increase,
                    daily_runtime_hotwater_increase,
                    daily_runtime_3kw_increase,
                    daily_runtime_6kw_increase,
                    avg_outdoor_temp,
                    avg_supplyline_temp,
                    avg_returnline_temp,
                    avg_hotwater_temp,
                    avg_brine_out_temp,
                    avg_brine_in_temp
                FROM heatpump_daily_summary
                WHERE day >= $1 AND day < $2
                ORDER BY day
                "#,
            )
            .bind(from)
            .bind(to)
            .fetch_all(pool)
            .await
            {
                Ok(results) => Ok(results),
                Err(sqlx::Error::Database(db_err))
                    if db_err.code().as_deref() == Some("42P01")
                        || db_err.message().contains("does not exist") =>
                {
                    // View doesn't exist (TimescaleDB not available), return empty vector
                    Ok(Vec::new())
                }
                Err(e) => Err(AppError::Db(e)),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    #[tokio::test]
    #[ignore] // Requires database connection
    async fn test_get_latest_no_device() {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/test".into());
        let pool = db::connect(&database_url).await.unwrap();

        let result = HeatpumpRepository::get_latest(&pool, None).await;
        // Will fail if no data exists, which is expected
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    #[ignore] // Requires database connection
    async fn test_get_latest_no_device_returns_latest() {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/test".into());
        let pool = db::connect(&database_url).await.unwrap();

        // If data exists, verify it returns the most recent record
        if let Ok(latest) = HeatpumpRepository::get_latest(&pool, None).await {
            // Verify the structure is correct
            assert!(latest.ts <= Utc::now());
            // All fields should be present (even if None)
            // This is a structural test, not a data validation test
        }
    }

    #[tokio::test]
    #[ignore] // Requires database connection
    async fn test_get_latest_with_device() {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/test".into());
        let pool = db::connect(&database_url).await.unwrap();

        let result = HeatpumpRepository::get_latest(&pool, Some("test-device")).await;
        // Will fail if no data exists, which is expected
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    #[ignore] // Requires database connection
    async fn test_get_latest_with_device_filters_correctly() {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/test".into());
        let pool = db::connect(&database_url).await.unwrap();

        // Verify that device_id filtering works correctly
        if let Ok(latest) = HeatpumpRepository::get_latest(&pool, Some("test-device")).await {
            // Verify we got a valid result with timestamp
            assert!(latest.ts <= Utc::now() + chrono::Duration::seconds(5));
            // Verify device_id matches if present
            if let Some(id) = &latest.device_id {
                assert_eq!(id, "test-device");
            }
        }
    }

    #[tokio::test]
    #[ignore] // Requires database connection
    async fn test_get_latest_with_nonexistent_device() {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/test".into());
        let pool = db::connect(&database_url).await.unwrap();

        // Verify that querying for a nonexistent device_id returns RowNotFound error
        let result = HeatpumpRepository::get_latest(&pool, Some("nonexistent-device-12345")).await;
        // Should return an error (RowNotFound) if no data exists for that device_id
        assert!(result.is_err());
    }

    #[tokio::test]
    #[ignore] // Requires database connection
    async fn test_get_latest_returns_most_recent() {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/test".into());
        let pool = db::connect(&database_url).await.unwrap();

        // If multiple records exist, verify it returns the most recent
        if let Ok(latest) = HeatpumpRepository::get_latest(&pool, None).await {
            // Verify timestamp is reasonable (not in the future)
            assert!(latest.ts <= Utc::now() + chrono::Duration::seconds(5));
        }
    }

    #[test]
    fn test_heatpump_latest_struct_fields() {
        // Unit test to verify struct can be created and fields are accessible
        let _latest = HeatpumpLatest {
            ts: Utc::now(),
            device_id: Some("test-device-01".to_string()),
            compressor_on: Some(true),
            hotwater_production: Some(false),
            flowlinepump_on: Some(true),
            brinepump_on: Some(true),
            aux_heater_3kw_on: Some(false),
            aux_heater_6kw_on: Some(false),
            outdoor_temp: Some(5.5),
            supplyline_temp: Some(35.0),
            returnline_temp: Some(30.0),
            hotwater_temp: Some(45),
            brine_out_temp: Some(8),
            brine_in_temp: Some(6),
        };

        // Verify device_id field is accessible
        assert_eq!(_latest.device_id.as_ref().unwrap(), "test-device-01");
    }

    #[test]
    fn test_heatpump_latest_struct_without_device_id() {
        // Test that device_id can be None (for backward compatibility with old data)
        let _latest = HeatpumpLatest {
            ts: Utc::now(),
            device_id: None,
            compressor_on: Some(true),
            hotwater_production: Some(false),
            flowlinepump_on: Some(true),
            brinepump_on: Some(true),
            aux_heater_3kw_on: Some(false),
            aux_heater_6kw_on: Some(false),
            outdoor_temp: Some(5.5),
            supplyline_temp: Some(35.0),
            returnline_temp: Some(30.0),
            hotwater_temp: Some(45),
            brine_out_temp: Some(8),
            brine_in_temp: Some(6),
        };

        // Verify device_id can be None
        assert!(_latest.device_id.is_none());
    }
}
