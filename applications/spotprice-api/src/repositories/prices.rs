use crate::db::DbPool;
use crate::error::AppError;
use crate::nordpool::PriceEntry;
use chrono::{DateTime, NaiveDate, Utc};
use sqlx::FromRow;

/// A stored spot price for one delivery period.
#[derive(Debug, Clone, FromRow)]
pub struct SpotPrice {
    pub time: DateTime<Utc>,
    pub price_per_kwh: f64,
    pub source_updated_at: Option<DateTime<Utc>>,
}

pub struct SpotPriceRepository;

impl SpotPriceRepository {
    /// Upsert all delivery periods for a day. Idempotent on (delivery_area, time).
    pub async fn upsert_day(
        pool: &DbPool,
        area: &str,
        currency: &str,
        entries: &[PriceEntry],
        source_updated_at: Option<DateTime<Utc>>,
        fetched_at: DateTime<Utc>,
    ) -> Result<u64, AppError> {
        let mut tx = pool.begin().await?;
        for entry in entries {
            let price_per_kwh = entry.price_per_mwh / 1000.0;
            sqlx::query(
                r#"
                INSERT INTO spot_prices
                    (time, delivery_area, currency, price_per_mwh, price_per_kwh,
                     source_updated_at, fetched_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                ON CONFLICT (delivery_area, time) DO UPDATE SET
                    currency = EXCLUDED.currency,
                    price_per_mwh = EXCLUDED.price_per_mwh,
                    price_per_kwh = EXCLUDED.price_per_kwh,
                    source_updated_at = EXCLUDED.source_updated_at,
                    fetched_at = EXCLUDED.fetched_at
                "#,
            )
            .bind(entry.start)
            .bind(area)
            .bind(currency)
            .bind(entry.price_per_mwh)
            .bind(price_per_kwh)
            .bind(source_updated_at)
            .bind(fetched_at)
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        Ok(entries.len() as u64)
    }

    /// Prices whose local (Europe/Stockholm) delivery date matches `date`, ordered by time.
    pub async fn get_for_local_date(
        pool: &DbPool,
        area: &str,
        date: NaiveDate,
    ) -> Result<Vec<SpotPrice>, AppError> {
        let rows = sqlx::query_as::<_, SpotPrice>(
            r#"
            SELECT time, price_per_kwh, source_updated_at
            FROM spot_prices
            WHERE delivery_area = $1
              AND (time AT TIME ZONE 'Europe/Stockholm')::date = $2
            ORDER BY time
            "#,
        )
        .bind(area)
        .bind(date)
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }

    /// Number of stored periods for a local delivery date (used for bootstrap checks).
    pub async fn count_for_local_date(
        pool: &DbPool,
        area: &str,
        date: NaiveDate,
    ) -> Result<i64, AppError> {
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM spot_prices
            WHERE delivery_area = $1
              AND (time AT TIME ZONE 'Europe/Stockholm')::date = $2
            "#,
        )
        .bind(area)
        .bind(date)
        .fetch_one(pool)
        .await?;
        Ok(count)
    }
}
