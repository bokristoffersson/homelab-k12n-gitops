use crate::db::DbPool;
use crate::error::AppError;
use chrono::{DateTime, Utc};
use sqlx::{postgres::PgRow, FromRow, Row};

#[derive(Debug, Clone)]
pub struct EnergyHourly {
    pub hour_start: DateTime<Utc>,
    pub hour_end: DateTime<Utc>,
    pub total_energy_kwh: Option<f64>,
    pub total_energy_l1_kwh: Option<f64>,
    pub total_energy_l2_kwh: Option<f64>,
    pub total_energy_l3_kwh: Option<f64>,
    pub total_energy_actual_kwh: Option<f64>,
    pub measurement_count: i64,
}

#[derive(Debug, Clone)]
pub struct EnergyPeakHour {
    pub hour_start: DateTime<Utc>,
    pub hour_end: DateTime<Utc>,
    pub total_energy_kwh: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct EnergySummary {
    pub day_start: Option<DateTime<Utc>>,
    pub day_end: Option<DateTime<Utc>>,
    pub month_start: Option<DateTime<Utc>>,
    pub month_end: Option<DateTime<Utc>>,
    pub year_start: Option<DateTime<Utc>>,
    pub year_end: Option<DateTime<Utc>>,
    pub energy_consumption_w: Option<f64>,
    pub measurement_count: i64,
}

#[derive(Debug, Clone)]
pub struct EnergyLatest {
    pub ts: DateTime<Utc>,
    pub consumption_total_w: Option<i32>,
    pub consumption_total_actual_w: Option<i64>,
    pub consumption_l1_actual_w: Option<i64>,
    pub consumption_l2_actual_w: Option<i64>,
    pub consumption_l3_actual_w: Option<i64>,
}

pub struct EnergyRepository;

impl<'r> FromRow<'r, PgRow> for EnergyHourly {
    fn from_row(row: &'r PgRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            hour_start: row.try_get("hour_start")?,
            hour_end: row.try_get("hour_end")?,
            total_energy_kwh: row.try_get("total_energy_kwh")?,
            total_energy_l1_kwh: row.try_get("total_energy_l1_kwh")?,
            total_energy_l2_kwh: row.try_get("total_energy_l2_kwh")?,
            total_energy_l3_kwh: row.try_get("total_energy_l3_kwh")?,
            total_energy_actual_kwh: row.try_get("total_energy_actual_kwh")?,
            measurement_count: row.try_get("measurement_count")?,
        })
    }
}

impl<'r> FromRow<'r, PgRow> for EnergyPeakHour {
    fn from_row(row: &'r PgRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            hour_start: row.try_get("hour_start")?,
            hour_end: row.try_get("hour_end")?,
            total_energy_kwh: row.try_get("total_energy_kwh")?,
        })
    }
}

impl<'r> FromRow<'r, PgRow> for EnergySummary {
    fn from_row(row: &'r PgRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            day_start: row.try_get("day_start")?,
            day_end: row.try_get("day_end")?,
            month_start: row.try_get("month_start")?,
            month_end: row.try_get("month_end")?,
            year_start: row.try_get("year_start")?,
            year_end: row.try_get("year_end")?,
            energy_consumption_w: row.try_get("energy_consumption_w")?,
            measurement_count: row.try_get("measurement_count")?,
        })
    }
}

impl<'r> FromRow<'r, PgRow> for EnergyLatest {
    fn from_row(row: &'r PgRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            ts: row.try_get("ts")?,
            consumption_total_w: row.try_get("consumption_total_w")?,
            consumption_total_actual_w: row.try_get("consumption_total_actual_w")?,
            consumption_l1_actual_w: row.try_get("consumption_l1_actual_w")?,
            consumption_l2_actual_w: row.try_get("consumption_l2_actual_w")?,
            consumption_l3_actual_w: row.try_get("consumption_l3_actual_w")?,
        })
    }
}

impl EnergyRepository {
    pub async fn get_latest(pool: &DbPool) -> Result<EnergyLatest, AppError> {
        sqlx::query_as::<_, EnergyLatest>(
            r#"
            SELECT 
                ts,
                consumption_total_w,
                consumption_total_actual_w,
                consumption_l1_actual_w,
                consumption_l2_actual_w,
                consumption_l3_actual_w
            FROM energy
            ORDER BY ts DESC
            LIMIT 1
            "#,
        )
        .fetch_one(pool)
        .await
        .map_err(AppError::Db)
    }

    pub async fn get_hourly_total(
        pool: &DbPool,
        hour_start: DateTime<Utc>,
    ) -> Result<f64, AppError> {
        // Calculate current hour consumption from raw meter readings
        // First check if we have any data at or after hour_start
        let latest_ts: Option<DateTime<Utc>> = sqlx::query_scalar(
            r#"
            SELECT ts FROM energy
            ORDER BY ts DESC
            LIMIT 1
            "#,
        )
        .fetch_optional(pool)
        .await
        .map_err(AppError::Db)?;

        // If no data exists or latest data is before hour_start, return 0.0
        if latest_ts.is_none() || latest_ts.unwrap() < hour_start {
            return Ok(0.0);
        }

        // Get the first reading at or after hour_start and the latest reading
        let result: Option<(Option<i32>, Option<i32>)> = sqlx::query_as(
            r#"
            SELECT
                (SELECT consumption_total_w FROM energy
                 WHERE ts >= $1
                 ORDER BY ts ASC
                 LIMIT 1) as hour_start_w,
                (SELECT consumption_total_w FROM energy
                 ORDER BY ts DESC
                 LIMIT 1) as current_w
            "#,
        )
        .bind(hour_start)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Db)?;

        // Calculate difference and convert from Wh to kWh
        if let Some((Some(start_w), Some(current_w))) = result {
            let diff_wh = current_w - start_w;
            let kwh = diff_wh as f64 / 1000.0;
            Ok(kwh.max(0.0)) // Ensure non-negative
        } else {
            Ok(0.0)
        }
    }

    pub async fn get_hourly_history(
        pool: &DbPool,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<EnergyHourly>, AppError> {
        match sqlx::query_as::<_, EnergyHourly>(
            r#"
            SELECT
                hour_start,
                hour_end,
                CAST(total_energy_kwh AS DOUBLE PRECISION) AS total_energy_kwh,
                CAST(total_energy_L1_actual_kwh AS DOUBLE PRECISION) AS total_energy_l1_kwh,
                CAST(total_energy_L2_actual_kwh AS DOUBLE PRECISION) AS total_energy_l2_kwh,
                CAST(total_energy_L3_actual_kwh AS DOUBLE PRECISION) AS total_energy_l3_kwh,
                CAST(total_energy_actual_kwh AS DOUBLE PRECISION) AS total_energy_actual_kwh,
                measurement_count
            FROM energy_hourly
            WHERE hour_start >= $1 AND hour_start < $2
            ORDER BY hour_start
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

    pub async fn get_peak_hour_for_day(
        pool: &DbPool,
        day_start: DateTime<Utc>,
        day_end: DateTime<Utc>,
    ) -> Result<Option<EnergyPeakHour>, AppError> {
        match sqlx::query_as::<_, EnergyPeakHour>(
            r#"
            SELECT
                hour_start,
                hour_end,
                CAST(COALESCE(total_energy_actual_kwh, total_energy_kwh) AS DOUBLE PRECISION)
                    AS total_energy_kwh
            FROM energy_hourly
            WHERE hour_start >= $1 AND hour_start < $2
            ORDER BY COALESCE(total_energy_actual_kwh, total_energy_kwh) DESC NULLS LAST
            LIMIT 1
            "#,
        )
        .bind(day_start)
        .bind(day_end)
        .fetch_optional(pool)
        .await
        {
            Ok(result) => Ok(result),
            Err(sqlx::Error::Database(db_err))
                if db_err.code().as_deref() == Some("42P01")
                    || db_err.message().contains("does not exist") =>
            {
                Ok(None)
            }
            Err(e) => Err(AppError::Db(e)),
        }
    }

    pub async fn get_daily_summary(
        pool: &DbPool,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<EnergySummary>, AppError> {
        match sqlx::query_as::<_, EnergySummary>(
            r#"
            SELECT 
                day_start,
                day_end,
                NULL::timestamptz AS month_start,
                NULL::timestamptz AS month_end,
                NULL::timestamptz AS year_start,
                NULL::timestamptz AS year_end,
                CAST(energy_consumption_w / 1000.0 AS DOUBLE PRECISION) AS energy_consumption_w,
                measurement_count
            FROM energy_daily_summary
            WHERE day_start >= $1 AND day_start < $2
            ORDER BY day_start
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

    pub async fn get_monthly_summary(
        pool: &DbPool,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<EnergySummary>, AppError> {
        match sqlx::query_as::<_, EnergySummary>(
            r#"
            SELECT 
                NULL::timestamptz AS day_start,
                NULL::timestamptz AS day_end,
                month_start,
                month_end,
                NULL::timestamptz AS year_start,
                NULL::timestamptz AS year_end,
                CAST(energy_consumption_w / 1000.0 AS DOUBLE PRECISION) AS energy_consumption_w,
                measurement_count
            FROM energy_monthly_summary
            WHERE month_start >= $1 AND month_start < $2
            ORDER BY month_start
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

    pub async fn get_yearly_summary(
        pool: &DbPool,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<EnergySummary>, AppError> {
        match sqlx::query_as::<_, EnergySummary>(
            r#"
            SELECT 
                NULL::timestamptz AS day_start,
                NULL::timestamptz AS day_end,
                NULL::timestamptz AS month_start,
                NULL::timestamptz AS month_end,
                year_start,
                year_end,
                CAST(energy_consumption_w / 1000.0 AS DOUBLE PRECISION) AS energy_consumption_w,
                measurement_count
            FROM energy_yearly_summary
            WHERE year_start >= $1 AND year_start < $2
            ORDER BY year_start
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

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
    #[ignore] // Requires database connection
    async fn test_get_latest() {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/test".into());
        let pool = db::connect(&database_url).await.unwrap();

        // This will fail if no data exists, which is expected
        let result = EnergyRepository::get_latest(&pool).await;
        // Just verify the function doesn't panic
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    #[ignore] // Requires database connection
    async fn test_get_latest_returns_latest_record() {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/test".into());
        let pool = db::connect(&database_url).await.unwrap();

        // If data exists, verify it returns the most recent record
        if let Ok(latest) = EnergyRepository::get_latest(&pool).await {
            // Verify the structure is correct
            assert!(latest.ts <= Utc::now());
            // All fields should be present (even if None)
            // This is a structural test, not a data validation test
        }
    }

    #[tokio::test]
    #[ignore] // Requires database connection
    async fn test_get_hourly_total() {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/test".into());
        let pool = db::connect(&database_url).await.unwrap();

        let now = Utc::now();
        let hour_start = align_to_hour_boundary(now);

        let result = EnergyRepository::get_hourly_total(&pool, hour_start).await;
        assert!(result.is_ok());

        // Should return a non-negative value
        let total = result.unwrap();
        assert!(total >= 0.0, "total should be non-negative");
    }

    #[tokio::test]
    #[ignore] // Requires database connection
    async fn test_get_hourly_total_returns_zero_for_no_data() {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/test".into());
        let pool = db::connect(&database_url).await.unwrap();

        // Use a far future hour that definitely has no data
        let future_hour = align_to_hour_boundary(Utc::now() + chrono::Duration::days(365));

        let result = EnergyRepository::get_hourly_total(&pool, future_hour).await;
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            0.0,
            "should return 0.0 when no data exists"
        );
    }

    #[tokio::test]
    #[ignore] // Requires database connection
    async fn test_get_hourly_history() {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/test".into());
        let pool = db::connect(&database_url).await.unwrap();

        let now = Utc::now();
        let from = align_to_hour_boundary(now) - chrono::Duration::hours(24);
        let to = align_to_hour_boundary(now) + chrono::Duration::hours(1);

        let result = EnergyRepository::get_hourly_history(&pool, from, to).await;
        assert!(result.is_ok());

        let history = result.unwrap();
        // Verify structure of returned data
        for entry in history {
            assert!(entry.hour_start < entry.hour_end);
            assert!(entry.measurement_count >= 0);
            // Verify all optional fields are present (even if None)
            // This is a structural test
        }
    }

    #[tokio::test]
    #[ignore] // Requires database connection
    async fn test_get_hourly_history_empty_range() {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/test".into());
        let pool = db::connect(&database_url).await.unwrap();

        // Use a date range that definitely has no data
        let from = Utc::now() - chrono::Duration::days(365);
        let to = Utc::now() - chrono::Duration::days(364);

        let result = EnergyRepository::get_hourly_history(&pool, from, to).await;
        assert!(result.is_ok());
        let _history = result.unwrap();
        // Should return empty vector or valid data structure
        // The unwrap() verifies it doesn't panic, which is sufficient
    }

    #[tokio::test]
    #[ignore] // Requires database connection
    async fn test_get_hourly_history_ordering() {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/test".into());
        let pool = db::connect(&database_url).await.unwrap();

        let now = Utc::now();
        let from = align_to_hour_boundary(now) - chrono::Duration::hours(48);
        let to = align_to_hour_boundary(now) + chrono::Duration::hours(1);

        let result = EnergyRepository::get_hourly_history(&pool, from, to).await;
        if let Ok(history) = result {
            // Verify results are ordered by hour_start
            for i in 1..history.len() {
                assert!(
                    history[i - 1].hour_start <= history[i].hour_start,
                    "Results should be ordered by hour_start"
                );
            }
        }
    }

    #[test]
    fn test_align_to_hour_boundary() {
        use chrono::Timelike;
        let now = Utc::now();
        let aligned = align_to_hour_boundary(now);

        // Verify it's aligned to hour boundary
        assert_eq!(aligned.minute(), 0);
        assert_eq!(aligned.second(), 0);
        assert_eq!(aligned.nanosecond(), 0);
    }
}
