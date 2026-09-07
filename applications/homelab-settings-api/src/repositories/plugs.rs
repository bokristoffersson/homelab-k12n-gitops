use chrono::{DateTime, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Postgres, Transaction};

use crate::error::{AppError, Result};

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct PowerPlug {
    pub plug_id: String,
    pub name: String,
    pub status: bool,
    pub wifi_rssi: Option<i32>,
    pub uptime_seconds: Option<i32>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PowerPlugCreate {
    pub plug_id: String,
    pub name: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PowerPlugUpdate {
    pub name: Option<String>,
}

/// Telemetry data received from Kafka (used in Phase 5)
#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct PowerPlugTelemetry {
    pub plug_id: String,
    pub status: bool,
    pub wifi_rssi: Option<i32>,
    pub uptime_seconds: Option<i32>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PowerPlugToggle {
    pub status: bool,
}

pub struct PlugsRepository {
    pool: PgPool,
}

impl PlugsRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Get all power plugs
    pub async fn get_all(&self) -> Result<Vec<PowerPlug>> {
        let plugs = sqlx::query_as::<_, PowerPlug>(
            r#"
            SELECT plug_id, name, status, wifi_rssi, uptime_seconds, updated_at
            FROM power_plugs
            ORDER BY name
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(plugs)
    }

    /// Get a power plug by ID
    pub async fn get_by_id(&self, plug_id: &str) -> Result<PowerPlug> {
        let plug = sqlx::query_as::<_, PowerPlug>(
            r#"
            SELECT plug_id, name, status, wifi_rssi, uptime_seconds, updated_at
            FROM power_plugs
            WHERE plug_id = $1
            "#,
        )
        .bind(plug_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Plug {} not found", plug_id)))?;

        Ok(plug)
    }

    /// Create a new power plug
    pub async fn create(&self, create: &PowerPlugCreate) -> Result<PowerPlug> {
        let plug = sqlx::query_as::<_, PowerPlug>(
            r#"
            INSERT INTO power_plugs (plug_id, name, status, updated_at)
            VALUES ($1, $2, false, NOW())
            RETURNING plug_id, name, status, wifi_rssi, uptime_seconds, updated_at
            "#,
        )
        .bind(&create.plug_id)
        .bind(&create.name)
        .fetch_one(&self.pool)
        .await?;

        Ok(plug)
    }

    /// Update a power plug's name
    pub async fn update(&self, plug_id: &str, update: &PowerPlugUpdate) -> Result<PowerPlug> {
        let mut query = String::from("UPDATE power_plugs SET updated_at = NOW()");

        if let Some(ref name) = update.name {
            query.push_str(", name = $2");
            query.push_str(" WHERE plug_id = $1 RETURNING plug_id, name, status, wifi_rssi, uptime_seconds, updated_at");

            let plug = sqlx::query_as::<_, PowerPlug>(&query)
                .bind(plug_id)
                .bind(name)
                .fetch_optional(&self.pool)
                .await?
                .ok_or_else(|| AppError::NotFound(format!("Plug {} not found", plug_id)))?;

            return Ok(plug);
        }

        // No fields to update, just return the existing plug
        self.get_by_id(plug_id).await
    }

    /// Update plug status within a transaction (for outbox pattern).
    /// Touches only the columns in PowerPlug; the desired-state bookkeeping
    /// is a separate set_desired_in_tx call in the same transaction.
    pub async fn update_status_in_tx(
        tx: &mut Transaction<'_, Postgres>,
        plug_id: &str,
        status: bool,
    ) -> Result<PowerPlug> {
        let plug = sqlx::query_as::<_, PowerPlug>(
            r#"
            UPDATE power_plugs
            SET status = $2, updated_at = NOW()
            WHERE plug_id = $1
            RETURNING plug_id, name, status, wifi_rssi, uptime_seconds, updated_at
            "#,
        )
        .bind(plug_id)
        .bind(status)
        .fetch_optional(&mut **tx)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Plug {} not found", plug_id)))?;

        Ok(plug)
    }

    /// Record the desired state without touching the reported status (used by
    /// the scheduler, which unlike the manual toggle never updated status
    /// optimistically)
    pub async fn set_desired_in_tx(
        tx: &mut Transaction<'_, Postgres>,
        plug_id: &str,
        status: bool,
    ) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE power_plugs
            SET desired_status = $2, desired_updated_at = NOW()
            WHERE plug_id = $1
            "#,
        )
        .bind(plug_id)
        .bind(status)
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    /// Adopt an externally caused state change as the new desired state.
    ///
    /// Homebridge (HomeKit/Siri) publishes plug commands directly to MQTT,
    /// bypassing this API. When the reported state disagrees with desired and
    /// no recent command explains the gap, the change was external and the
    /// user's latest intent wins — otherwise the reconciler would fight Siri
    /// and flip the plug back.
    ///
    /// What blocks adoption depends on the echo type (`is_transition`):
    /// - pending/published commands matching desired always block — the echo
    ///   may predate a command still in flight.
    /// - failed commands block only periodic echoes (tele/STATE reporting an
    ///   unchanged state): that echo is just the state the device was stuck
    ///   at, and adopting it would cancel reconciliation of the lost command.
    ///   A transition echo (stat/POWER fires only when the state actually
    ///   changed) means someone actuated the plug — Siri, button, HomeKit —
    ///   and that overrides even a failed command's intent.
    ///
    /// The 15-minute window comfortably covers the confirmation loop's whole
    /// retry chain (initial publish + max_retries republishes at
    /// CONFIRM_TIMEOUT_SECS = 90s, roughly 6 minutes worst case) and the
    /// reconciler keeps it refreshed with its own attempts while a mismatch
    /// persists. Revisit if those timings change.
    pub async fn adopt_external_state(
        &self,
        plug_id: &str,
        reported: bool,
        is_transition: bool,
    ) -> Result<bool> {
        let result = sqlx::query(
            r#"
            UPDATE power_plugs p
            SET desired_status = $2, desired_updated_at = NOW()
            WHERE p.plug_id = $1
              AND p.desired_status IS NOT NULL
              AND p.desired_status IS DISTINCT FROM $2
              AND NOT EXISTS (
                  SELECT 1 FROM outbox o
                  WHERE o.aggregate_type = 'power_plug'
                    AND o.aggregate_id = p.plug_id
                    AND (
                        o.status IN ('pending', 'published')
                        OR ($3 = false AND o.status = 'failed')
                    )
                    AND o.created_at > NOW() - INTERVAL '15 minutes'
                    AND o.payload->>'action' =
                        CASE WHEN p.desired_status THEN 'ON' ELSE 'OFF' END
              )
            "#,
        )
        .bind(plug_id)
        .bind(reported)
        .bind(is_transition)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Plugs whose reported state disagrees with the desired state and need a
    /// reconcile command. Excludes plugs with a command already in flight and
    /// rate-limits to one new attempt per backoff window.
    pub async fn get_state_mismatches(
        &self,
        grace_secs: u64,
        backoff_secs: u64,
    ) -> Result<Vec<(String, bool)>> {
        let rows = sqlx::query_as::<_, (String, bool)>(
            r#"
            SELECT p.plug_id, p.desired_status
            FROM power_plugs p
            WHERE p.desired_status IS NOT NULL
              AND p.desired_status IS DISTINCT FROM p.status
              AND p.desired_updated_at < NOW() - ($1 * INTERVAL '1 second')
              AND NOT EXISTS (
                  SELECT 1 FROM outbox o
                  WHERE o.aggregate_type = 'power_plug'
                    AND o.aggregate_id = p.plug_id
                    AND o.status IN ('pending', 'published')
              )
              AND NOT EXISTS (
                  -- backoff counts actual attempts; confirmed/superseded rows
                  -- must not suppress a reconcile of a fresh mismatch
                  SELECT 1 FROM outbox o
                  WHERE o.aggregate_type = 'power_plug'
                    AND o.aggregate_id = p.plug_id
                    AND o.status IN ('pending', 'published', 'failed')
                    AND o.created_at > NOW() - ($2 * INTERVAL '1 second')
              )
            ORDER BY p.plug_id
            "#,
        )
        .bind(grace_secs as i64)
        .bind(backoff_secs as i64)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    /// Upsert plug telemetry data (used by Kafka consumer in Phase 5)
    #[allow(dead_code)]
    pub async fn upsert_telemetry(&self, telemetry: &PowerPlugTelemetry) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO power_plugs (plug_id, name, status, wifi_rssi, uptime_seconds, updated_at)
            VALUES ($1, $1, $2, $3, $4, NOW())
            ON CONFLICT (plug_id) DO UPDATE SET
                status = EXCLUDED.status,
                wifi_rssi = COALESCE(EXCLUDED.wifi_rssi, power_plugs.wifi_rssi),
                uptime_seconds = COALESCE(EXCLUDED.uptime_seconds, power_plugs.uptime_seconds),
                updated_at = NOW()
            "#,
        )
        .bind(&telemetry.plug_id)
        .bind(telemetry.status)
        .bind(telemetry.wifi_rssi)
        .bind(telemetry.uptime_seconds)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Delete a power plug
    pub async fn delete(&self, plug_id: &str) -> Result<()> {
        let result = sqlx::query("DELETE FROM power_plugs WHERE plug_id = $1")
            .bind(plug_id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(format!("Plug {} not found", plug_id)));
        }

        Ok(())
    }
}

// Schedule-related types and repository
#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct PowerPlugSchedule {
    pub id: i64,
    pub plug_id: String,
    pub action: String,
    #[serde(with = "time_format")]
    pub time_of_day: NaiveTime,
    pub enabled: bool,
    pub last_fired_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ScheduleCreate {
    pub action: String,
    #[serde(with = "time_format")]
    pub time_of_day: NaiveTime,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_enabled() -> bool {
    true
}

#[derive(Debug, Deserialize, Clone)]
pub struct ScheduleUpdate {
    pub action: Option<String>,
    #[serde(default, with = "option_time_format")]
    pub time_of_day: Option<NaiveTime>,
    pub enabled: Option<bool>,
}

// Custom serde module for NaiveTime (HH:MM:SS or HH:MM format)
mod time_format {
    use chrono::NaiveTime;
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(time: &NaiveTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = time.format("%H:%M:%S").to_string();
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        // Try HH:MM:SS first, then HH:MM
        NaiveTime::parse_from_str(&s, "%H:%M:%S")
            .or_else(|_| NaiveTime::parse_from_str(&s, "%H:%M"))
            .map_err(serde::de::Error::custom)
    }
}

mod option_time_format {
    use chrono::NaiveTime;
    use serde::{self, Deserialize, Deserializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<NaiveTime>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt: Option<String> = Option::deserialize(deserializer)?;
        match opt {
            Some(s) => NaiveTime::parse_from_str(&s, "%H:%M:%S")
                .or_else(|_| NaiveTime::parse_from_str(&s, "%H:%M"))
                .map(Some)
                .map_err(serde::de::Error::custom),
            None => Ok(None),
        }
    }
}

pub struct SchedulesRepository {
    pool: PgPool,
}

impl SchedulesRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Get all schedules for a plug
    pub async fn get_by_plug_id(&self, plug_id: &str) -> Result<Vec<PowerPlugSchedule>> {
        let schedules = sqlx::query_as::<_, PowerPlugSchedule>(
            r#"
            SELECT id, plug_id, action, time_of_day, enabled, last_fired_at, created_at, updated_at
            FROM power_plug_schedules
            WHERE plug_id = $1
            ORDER BY time_of_day
            "#,
        )
        .bind(plug_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(schedules)
    }

    /// Get a schedule by ID
    pub async fn get_by_id(&self, id: i64) -> Result<PowerPlugSchedule> {
        let schedule = sqlx::query_as::<_, PowerPlugSchedule>(
            r#"
            SELECT id, plug_id, action, time_of_day, enabled, last_fired_at, created_at, updated_at
            FROM power_plug_schedules
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Schedule {} not found", id)))?;

        Ok(schedule)
    }

    /// Create a new schedule for a plug
    pub async fn create(
        &self,
        plug_id: &str,
        create: &ScheduleCreate,
    ) -> Result<PowerPlugSchedule> {
        // Validate action
        if create.action != "on" && create.action != "off" {
            return Err(AppError::InvalidInput(
                "action must be 'on' or 'off'".to_string(),
            ));
        }

        let schedule = sqlx::query_as::<_, PowerPlugSchedule>(
            r#"
            INSERT INTO power_plug_schedules (plug_id, action, time_of_day, enabled, created_at, updated_at)
            VALUES ($1, $2, $3, $4, NOW(), NOW())
            RETURNING id, plug_id, action, time_of_day, enabled, last_fired_at, created_at, updated_at
            "#,
        )
        .bind(plug_id)
        .bind(&create.action)
        .bind(create.time_of_day)
        .bind(create.enabled)
        .fetch_one(&self.pool)
        .await?;

        Ok(schedule)
    }

    /// Update a schedule
    pub async fn update(&self, id: i64, update: &ScheduleUpdate) -> Result<PowerPlugSchedule> {
        // Validate action if provided
        if let Some(ref action) = update.action {
            if action != "on" && action != "off" {
                return Err(AppError::InvalidInput(
                    "action must be 'on' or 'off'".to_string(),
                ));
            }
        }

        let mut query = String::from("UPDATE power_plug_schedules SET updated_at = NOW()");
        let mut bind_count = 1;

        if update.action.is_some() {
            bind_count += 1;
            query.push_str(&format!(", action = ${}", bind_count));
        }
        if update.time_of_day.is_some() {
            bind_count += 1;
            query.push_str(&format!(", time_of_day = ${}", bind_count));
        }
        if update.enabled.is_some() {
            bind_count += 1;
            query.push_str(&format!(", enabled = ${}", bind_count));
        }

        query.push_str(" WHERE id = $1 RETURNING id, plug_id, action, time_of_day, enabled, last_fired_at, created_at, updated_at");

        let mut query_builder = sqlx::query_as::<_, PowerPlugSchedule>(&query).bind(id);

        if let Some(ref action) = update.action {
            query_builder = query_builder.bind(action);
        }
        if let Some(time) = update.time_of_day {
            query_builder = query_builder.bind(time);
        }
        if let Some(enabled) = update.enabled {
            query_builder = query_builder.bind(enabled);
        }

        let schedule = query_builder
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Schedule {} not found", id)))?;

        Ok(schedule)
    }

    /// Delete a schedule
    pub async fn delete(&self, id: i64) -> Result<()> {
        let result = sqlx::query("DELETE FROM power_plug_schedules WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(format!("Schedule {} not found", id)));
        }

        Ok(())
    }

    /// Get enabled schedules whose time_of_day has passed today (local time)
    /// but have not fired since today's scheduled instant. Unlike an
    /// exact-minute window this catches up after tick drift or pod restarts:
    /// a schedule missed at 18:00 still fires when the next tick runs.
    ///
    /// The updated_at guard keeps a schedule created or edited *after* its
    /// time has already passed today from firing retroactively — it starts
    /// tomorrow instead.
    ///
    /// `local_midnight_utc + time_of_day` is today's scheduled instant. On the
    /// two DST transition days per year this is off by one hour, which only
    /// shifts the already-fired-today comparison, not the firing time.
    pub async fn get_due_schedules(
        &self,
        current_local_time: NaiveTime,
        local_midnight_utc: DateTime<Utc>,
    ) -> Result<Vec<PowerPlugSchedule>> {
        let schedules = sqlx::query_as::<_, PowerPlugSchedule>(
            r#"
            SELECT id, plug_id, action, time_of_day, enabled, last_fired_at, created_at, updated_at
            FROM power_plug_schedules
            WHERE enabled = true
              AND time_of_day <= $1
              AND updated_at < ($2 + time_of_day::interval)
              AND (last_fired_at IS NULL OR last_fired_at < ($2 + time_of_day::interval))
            ORDER BY time_of_day ASC
            "#,
        )
        .bind(current_local_time)
        .bind(local_midnight_utc)
        .fetch_all(&self.pool)
        .await?;

        Ok(schedules)
    }

    /// Record that a schedule fired, within the same transaction as the
    /// outbox insert so a crash cannot fire it twice or lose the marker
    pub async fn mark_fired_in_tx(tx: &mut Transaction<'_, Postgres>, id: i64) -> Result<()> {
        sqlx::query("UPDATE power_plug_schedules SET last_fired_at = NOW() WHERE id = $1")
            .bind(id)
            .execute(&mut **tx)
            .await?;

        Ok(())
    }
}

// DB-backed tests for the reconciler/adoption query logic. They run only when
// TEST_DATABASE_URL points at a PostgreSQL instance (CI provides a service
// container); without it the test skips so plain `cargo test` stays green.
#[cfg(test)]
mod db_tests {
    use super::*;
    use sqlx::postgres::PgPoolOptions;

    async fn test_pool() -> Option<PgPool> {
        let url = std::env::var("TEST_DATABASE_URL").ok()?;
        PgPoolOptions::new()
            .max_connections(2)
            .connect(&url)
            .await
            .ok()
    }

    /// Minimal mirror of migrations 003/006/008 — only the columns the
    /// queries under test touch
    async fn setup_schema(pool: &PgPool) {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS power_plugs (
                plug_id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                status BOOLEAN DEFAULT false,
                wifi_rssi INTEGER,
                uptime_seconds INTEGER,
                desired_status BOOLEAN,
                desired_updated_at TIMESTAMPTZ,
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
            "#,
        )
        .execute(pool)
        .await
        .unwrap();

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS outbox (
                id BIGSERIAL PRIMARY KEY,
                aggregate_type VARCHAR(255) NOT NULL,
                aggregate_id VARCHAR(255) NOT NULL,
                event_type VARCHAR(255) NOT NULL,
                payload JSONB NOT NULL,
                status VARCHAR(50) NOT NULL DEFAULT 'pending',
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                published_at TIMESTAMPTZ,
                confirmed_at TIMESTAMPTZ,
                error_message TEXT,
                retry_count INT NOT NULL DEFAULT 0,
                max_retries INT NOT NULL DEFAULT 3
            )
            "#,
        )
        .execute(pool)
        .await
        .unwrap();

        sqlx::query("TRUNCATE power_plugs, outbox")
            .execute(pool)
            .await
            .unwrap();
    }

    async fn insert_plug(
        pool: &PgPool,
        plug_id: &str,
        status: bool,
        desired: Option<bool>,
        desired_age_secs: i64,
    ) {
        sqlx::query(
            r#"
            INSERT INTO power_plugs
                (plug_id, name, status, desired_status, desired_updated_at, updated_at)
            VALUES ($1, $1, $2, $3, NOW() - ($4 * INTERVAL '1 second'), NOW())
            "#,
        )
        .bind(plug_id)
        .bind(status)
        .bind(desired)
        .bind(desired_age_secs)
        .execute(pool)
        .await
        .unwrap();
    }

    async fn insert_command(
        pool: &PgPool,
        plug_id: &str,
        action: &str,
        outbox_status: &str,
        age_secs: i64,
    ) {
        sqlx::query(
            r#"
            INSERT INTO outbox
                (aggregate_type, aggregate_id, event_type, payload, status, created_at)
            VALUES ('power_plug', $1, 'plug_toggle',
                    jsonb_build_object('plug_id', $1, 'action', $2),
                    $3, NOW() - ($4 * INTERVAL '1 second'))
            "#,
        )
        .bind(plug_id)
        .bind(action)
        .bind(outbox_status)
        .bind(age_secs)
        .execute(pool)
        .await
        .unwrap();
    }

    async fn desired_of(pool: &PgPool, plug_id: &str) -> Option<bool> {
        sqlx::query_scalar("SELECT desired_status FROM power_plugs WHERE plug_id = $1")
            .bind(plug_id)
            .fetch_one(pool)
            .await
            .unwrap()
    }

    const GRACE: u64 = 120;
    const BACKOFF: u64 = 300;

    #[tokio::test]
    async fn state_mismatch_and_adoption_query_logic() {
        let Some(pool) = test_pool().await else {
            eprintln!("TEST_DATABASE_URL not set, skipping DB tests");
            return;
        };
        setup_schema(&pool).await;
        let repo = PlugsRepository::new(pool.clone());

        // Plain mismatch past the grace period, no commands -> reconciled
        insert_plug(&pool, "p_mismatch", false, Some(true), 600).await;
        // Mismatch but desired changed recently -> grace suppresses it
        insert_plug(&pool, "p_grace", false, Some(true), 10).await;
        // Mismatch with a command in flight -> suppressed
        insert_plug(&pool, "p_inflight", false, Some(true), 600).await;
        insert_command(&pool, "p_inflight", "ON", "published", 30).await;
        // Mismatch with a fresh failed attempt -> backoff suppresses it
        insert_plug(&pool, "p_backoff", false, Some(true), 600).await;
        insert_command(&pool, "p_backoff", "ON", "failed", 60).await;
        // Mismatch with an old failed attempt -> backoff has passed
        insert_plug(&pool, "p_retry", false, Some(true), 600).await;
        insert_command(&pool, "p_retry", "ON", "failed", 600).await;
        // Fresh confirmed command must NOT suppress a new mismatch
        insert_plug(&pool, "p_confirmed", false, Some(true), 600).await;
        insert_command(&pool, "p_confirmed", "OFF", "confirmed", 60).await;
        // No desired state recorded -> reconciler ignores the plug
        insert_plug(&pool, "p_null", false, None, 600).await;
        // Converged plug -> nothing to do
        insert_plug(&pool, "p_ok", true, Some(true), 600).await;

        let ids: Vec<String> = repo
            .get_state_mismatches(GRACE, BACKOFF)
            .await
            .unwrap()
            .into_iter()
            .map(|(id, _)| id)
            .collect();

        assert!(ids.contains(&"p_mismatch".to_string()));
        assert!(ids.contains(&"p_retry".to_string()));
        assert!(
            ids.contains(&"p_confirmed".to_string()),
            "a fresh confirmed command must not suppress reconciliation"
        );
        assert!(!ids.contains(&"p_grace".to_string()));
        assert!(!ids.contains(&"p_inflight".to_string()));
        assert!(
            !ids.contains(&"p_backoff".to_string()),
            "a fresh failed attempt must back off reconciliation"
        );
        assert!(!ids.contains(&"p_null".to_string()));
        assert!(!ids.contains(&"p_ok".to_string()));

        // Adoption: periodic (tele) echo of the stuck state must NOT override
        // a recent failed command - that would cancel reconciliation
        insert_plug(&pool, "p_adopt_tele", true, Some(true), 600).await;
        insert_command(&pool, "p_adopt_tele", "ON", "failed", 60).await;
        assert!(!repo
            .adopt_external_state("p_adopt_tele", false, false)
            .await
            .unwrap());
        assert_eq!(desired_of(&pool, "p_adopt_tele").await, Some(true));

        // ...but a transition (stat) echo means someone actuated the plug:
        // the user's latest intent wins even over a failed command
        insert_plug(&pool, "p_adopt_stat", true, Some(true), 600).await;
        insert_command(&pool, "p_adopt_stat", "ON", "failed", 60).await;
        assert!(repo
            .adopt_external_state("p_adopt_stat", false, true)
            .await
            .unwrap());
        assert_eq!(desired_of(&pool, "p_adopt_stat").await, Some(false));

        // An in-flight command blocks adoption regardless of echo type
        insert_plug(&pool, "p_adopt_inflight", true, Some(true), 600).await;
        insert_command(&pool, "p_adopt_inflight", "ON", "published", 5).await;
        assert!(!repo
            .adopt_external_state("p_adopt_inflight", false, true)
            .await
            .unwrap());

        // With no recent commands at all, even a tele echo adopts
        insert_plug(&pool, "p_adopt_free", true, Some(true), 600).await;
        assert!(repo
            .adopt_external_state("p_adopt_free", false, false)
            .await
            .unwrap());
        assert_eq!(desired_of(&pool, "p_adopt_free").await, Some(false));

        // Echo matching desired is a no-op
        insert_plug(&pool, "p_adopt_match", false, Some(true), 600).await;
        assert!(!repo
            .adopt_external_state("p_adopt_match", true, true)
            .await
            .unwrap());
    }
}
