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

/// Get pending outbox entries. retry_count may equal max_retries here: a
/// command requeued by the confirmation timeout gets one final publish before
/// fail_unconfirmed_plug_commands marks it failed.
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
          AND retry_count <= max_retries
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

/// Mark an outbox entry as confirmed (when telemetry received from device)
pub async fn mark_confirmed(pool: &Pool<Postgres>, aggregate_id: &str) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        r#"
        UPDATE outbox
        SET status = 'confirmed',
            confirmed_at = NOW()
        WHERE aggregate_type = 'heatpump_setting'
          AND aggregate_id = $1
          AND status = 'published'
          AND confirmed_at IS NULL
        "#,
    )
    .bind(aggregate_id)
    .execute(pool)
    .await?;
    Ok(result.rows_affected())
}

/// Mark plug commands as confirmed when the device echoes the matching state.
/// Matching on the echoed action (not just the plug id) ensures an unrelated
/// telemetry message cannot confirm a command that never took effect.
pub async fn mark_plug_confirmed(
    pool: &Pool<Postgres>,
    plug_id: &str,
    action: &str,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        r#"
        UPDATE outbox
        SET status = 'confirmed',
            confirmed_at = NOW()
        WHERE aggregate_type = 'power_plug'
          AND aggregate_id = $1
          AND status = 'published'
          AND payload->>'action' = $2
        "#,
    )
    .bind(plug_id)
    .bind(action)
    .execute(pool)
    .await?;
    Ok(result.rows_affected())
}

/// Fail plug commands that stayed unconfirmed after exhausting all republishes
pub async fn fail_unconfirmed_plug_commands(
    pool: &Pool<Postgres>,
    timeout_secs: u64,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        r#"
        UPDATE outbox
        SET status = 'failed',
            error_message = 'no device state confirmation after ' || retry_count || ' republish attempt(s)'
        WHERE aggregate_type = 'power_plug'
          AND status = 'published'
          AND published_at < NOW() - ($1 * INTERVAL '1 second')
          AND retry_count >= max_retries
        "#,
    )
    .bind(timeout_secs as i64)
    .execute(pool)
    .await?;
    Ok(result.rows_affected())
}

/// Requeue plug commands that were published but not confirmed by a device
/// state echo within the timeout, so the main loop republishes them
pub async fn requeue_unconfirmed_plug_commands(
    pool: &Pool<Postgres>,
    timeout_secs: u64,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        r#"
        UPDATE outbox
        SET status = 'pending',
            retry_count = retry_count + 1,
            error_message = 'no device state confirmation within timeout, republishing'
        WHERE aggregate_type = 'power_plug'
          AND status = 'published'
          AND published_at < NOW() - ($1 * INTERVAL '1 second')
          AND retry_count < max_retries
        "#,
    )
    .bind(timeout_secs as i64)
    .execute(pool)
    .await?;
    Ok(result.rows_affected())
}
