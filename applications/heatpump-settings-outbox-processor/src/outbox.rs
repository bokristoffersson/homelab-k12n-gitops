use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
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

/// Get pending outbox entries (status = 'pending', retry_count < max_retries)
pub async fn get_pending_entries(
    pool: &Pool<Postgres>,
    limit: i64,
) -> Result<Vec<OutboxEntry>, sqlx::Error> {
    sqlx::query_as::<_, OutboxEntry>(
        r#"
        SELECT id, aggregate_type, aggregate_id, event_type, payload, status,
               created_at, published_at, confirmed_at, error_message, retry_count, max_retries
        FROM outbox
        WHERE status = 'pending'
          AND retry_count < max_retries
        ORDER BY created_at ASC
        LIMIT $1
        "#,
    )
    .bind(limit)
    .fetch_all(pool)
    .await
}

/// Mark an outbox entry as published
pub async fn mark_published(pool: &Pool<Postgres>, id: i64) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE outbox
        SET status = 'published',
            published_at = NOW()
        WHERE id = $1
        "#,
    )
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

/// Mark an outbox entry as failed
pub async fn mark_failed(
    pool: &Pool<Postgres>,
    id: i64,
    error_message: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE outbox
        SET status = 'failed',
            error_message = $2
        WHERE id = $1
        "#,
    )
    .bind(id)
    .bind(error_message)
    .execute(pool)
    .await?;
    Ok(())
}

/// Increment retry count for an outbox entry
pub async fn increment_retry(
    pool: &Pool<Postgres>,
    id: i64,
    error_message: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE outbox
        SET retry_count = retry_count + 1,
            error_message = $2
        WHERE id = $1
        "#,
    )
    .bind(id)
    .bind(error_message)
    .execute(pool)
    .await?;
    Ok(())
}
