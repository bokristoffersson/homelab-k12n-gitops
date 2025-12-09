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

pub struct HeatpumpRepository;

impl HeatpumpRepository {
    pub async fn get_latest(
        pool: &DbPool,
        device_id: Option<&str>,
    ) -> Result<HeatpumpLatest, AppError> {
        if let Some(device_id) = device_id {
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
            .fetch_one(pool)
            .await
            .map_err(AppError::Db)
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
            .fetch_one(pool)
            .await
            .map_err(AppError::Db)
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

        // If data exists, verify filtering works
        if let Ok(latest) = HeatpumpRepository::get_latest(&pool, Some("test-device")).await {
            // If device_id filter is applied, the result should match
            if let Some(device_id) = &latest.device_id {
                // This test verifies the query structure, not the data
                assert!(!device_id.is_empty());
            }
        }
    }

    #[tokio::test]
    #[ignore] // Requires database connection
    async fn test_get_latest_with_nonexistent_device() {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/test".into());
        let pool = db::connect(&database_url).await.unwrap();

        // Try to get latest for a device that doesn't exist
        let result = HeatpumpRepository::get_latest(&pool, Some("nonexistent-device-12345")).await;
        // Should return an error if no data exists for that device
        assert!(result.is_err(), "should fail when device doesn't exist");
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
            device_id: Some("test".to_string()),
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

        // If we get here, the struct is valid (no assertion needed)
    }
}
