use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::models::Slot;

const SLOT_COLUMNS: &str =
    "id, starts_at, ends_at, note, booked_by_username, booked_by_email, booked_at, updated_at";

#[derive(Clone)]
pub struct SlotsRepository {
    pool: PgPool,
}

impl SlotsRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(
        &self,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<Slot>, sqlx::Error> {
        sqlx::query_as::<_, Slot>(&format!(
            "SELECT {SLOT_COLUMNS} FROM slots \
             WHERE starts_at >= $1 AND starts_at < $2 \
             ORDER BY starts_at"
        ))
        .bind(from)
        .bind(to)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn get(&self, id: i64) -> Result<Option<Slot>, sqlx::Error> {
        sqlx::query_as::<_, Slot>(&format!("SELECT {SLOT_COLUMNS} FROM slots WHERE id = $1"))
            .bind(id)
            .fetch_optional(&self.pool)
            .await
    }

    pub async fn create(
        &self,
        starts_at: DateTime<Utc>,
        ends_at: DateTime<Utc>,
        note: Option<&str>,
        created_by: &str,
    ) -> Result<Slot, sqlx::Error> {
        sqlx::query_as::<_, Slot>(&format!(
            "INSERT INTO slots (starts_at, ends_at, note, created_by) \
             VALUES ($1, $2, $3, $4) \
             RETURNING {SLOT_COLUMNS}"
        ))
        .bind(starts_at)
        .bind(ends_at)
        .bind(note)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn delete(&self, id: i64) -> Result<Option<Slot>, sqlx::Error> {
        sqlx::query_as::<_, Slot>(&format!(
            "DELETE FROM slots WHERE id = $1 RETURNING {SLOT_COLUMNS}"
        ))
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    /// Book a slot for a user. The WHERE clause makes double-booking race-safe:
    /// returns None when the slot is already booked, already started, or missing.
    pub async fn book(
        &self,
        id: i64,
        username: &str,
        email: Option<&str>,
    ) -> Result<Option<Slot>, sqlx::Error> {
        sqlx::query_as::<_, Slot>(&format!(
            "UPDATE slots SET booked_by_username = $2, booked_by_email = $3, \
             booked_at = NOW(), updated_at = NOW() \
             WHERE id = $1 AND booked_by_username IS NULL AND starts_at > NOW() \
             RETURNING {SLOT_COLUMNS}"
        ))
        .bind(id)
        .bind(username)
        .bind(email)
        .fetch_optional(&self.pool)
        .await
    }

    /// Clear a booking. Returns None when the slot is missing or not booked.
    pub async fn cancel(&self, id: i64) -> Result<Option<Slot>, sqlx::Error> {
        sqlx::query_as::<_, Slot>(&format!(
            "UPDATE slots SET booked_by_username = NULL, booked_by_email = NULL, \
             booked_at = NULL, updated_at = NOW() \
             WHERE id = $1 AND booked_by_username IS NOT NULL \
             RETURNING {SLOT_COLUMNS}"
        ))
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }
}
