use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Postgres, Transaction};

use crate::error::{AppError, Result};
use crate::repositories::settings::SettingPatch;

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct OutboxEntry {
    pub id: i64,
    pub aggregate_type: String,
    pub aggregate_id: String,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub published_at: Option<DateTime<Utc>>,
    pub confirmed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
    pub retry_count: i32,
    pub max_retries: i32,
}

#[derive(Debug, Clone)]
pub struct OutboxRepository {
    pool: PgPool,
}

impl OutboxRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Insert a new outbox command within an existing transaction
    pub async fn insert_in_tx(
        tx: &mut Transaction<'_, Postgres>,
        device_id: &str,
        patch: &SettingPatch,
    ) -> Result<OutboxEntry> {
        let payload = serde_json::to_value(patch)
            .map_err(|e| AppError::Internal(format!("Failed to serialize patch: {}", e)))?;

        let entry = sqlx::query_as::<_, OutboxEntry>(
            r#"
            INSERT INTO outbox (
                aggregate_type,
                aggregate_id,
                event_type,
                payload,
                status,
                created_at,
                retry_count,
                max_retries
            ) VALUES ($1, $2, $3, $4, $5, NOW(), $6, $7)
            RETURNING *
            "#,
        )
        .bind("heatpump_setting")
        .bind(device_id)
        .bind("setting_update")
        .bind(payload)
        .bind("pending")
        .bind(0) // retry_count
        .bind(3) // max_retries
        .fetch_one(&mut **tx)
        .await?;

        Ok(entry)
    }

    /// Get outbox entry by ID
    pub async fn get_by_id(&self, id: i64) -> Result<OutboxEntry> {
        let entry = sqlx::query_as::<_, OutboxEntry>(
            r#"
            SELECT * FROM outbox WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Outbox entry {} not found", id)))?;

        Ok(entry)
    }

    /// Get pending outbox entries (for processor)
    #[allow(dead_code)]
    pub async fn get_pending(&self, limit: i64) -> Result<Vec<OutboxEntry>> {
        let entries = sqlx::query_as::<_, OutboxEntry>(
            r#"
            SELECT * FROM outbox
            WHERE status = 'pending'
            ORDER BY created_at ASC
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(entries)
    }

    /// Update status to 'published'
    #[allow(dead_code)]
    pub async fn mark_published(&self, id: i64) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE outbox
            SET status = 'published', published_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Update status to 'confirmed'
    #[allow(dead_code)]
    pub async fn mark_confirmed(&self, id: i64) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE outbox
            SET status = 'confirmed', confirmed_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Update status to 'failed' with error message
    #[allow(dead_code)]
    pub async fn mark_failed(&self, id: i64, error: &str) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE outbox
            SET status = 'failed',
                error_message = $2,
                retry_count = retry_count + 1
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(error)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Increment retry count
    #[allow(dead_code)]
    pub async fn increment_retry(&self, id: i64) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE outbox
            SET retry_count = retry_count + 1
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get published entries awaiting confirmation (for processor)
    #[allow(dead_code)]
    pub async fn get_published_pending_confirmation(&self, limit: i64) -> Result<Vec<OutboxEntry>> {
        let entries = sqlx::query_as::<_, OutboxEntry>(
            r#"
            SELECT * FROM outbox
            WHERE status = 'published'
              AND published_at < NOW() - INTERVAL '60 seconds'
            ORDER BY published_at ASC
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(entries)
    }
}
