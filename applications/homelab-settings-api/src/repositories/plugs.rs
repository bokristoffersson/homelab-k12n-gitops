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
    /// Also records the desired state so the reconciler can re-issue the
    /// command if the device never converges.
    pub async fn update_status_in_tx(
        tx: &mut Transaction<'_, Postgres>,
        plug_id: &str,
        status: bool,
    ) -> Result<PowerPlug> {
        let plug = sqlx::query_as::<_, PowerPlug>(
            r#"
            UPDATE power_plugs
            SET status = $2,
                desired_status = $2,
                desired_updated_at = NOW(),
                updated_at = NOW()
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
    /// no recent unconfirmed command explains the gap, the change was external
    /// and the user's latest intent wins — otherwise the reconciler would
    /// fight Siri and flip the plug back. Recent pending/published/failed
    /// commands matching the desired action DO explain the gap (a command in
    /// flight or one the device never applied), so those block adoption and
    /// leave the reconciler in charge.
    pub async fn adopt_external_state(&self, plug_id: &str, reported: bool) -> Result<bool> {
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
                    AND o.status IN ('pending', 'published', 'failed')
                    AND o.created_at > NOW() - INTERVAL '15 minutes'
                    AND o.payload->>'action' =
                        CASE WHEN p.desired_status THEN 'ON' ELSE 'OFF' END
              )
            "#,
        )
        .bind(plug_id)
        .bind(reported)
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
              AND p.desired_updated_at < NOW() - make_interval(secs => $1::double precision)
              AND NOT EXISTS (
                  SELECT 1 FROM outbox o
                  WHERE o.aggregate_type = 'power_plug'
                    AND o.aggregate_id = p.plug_id
                    AND o.status IN ('pending', 'published')
              )
              AND NOT EXISTS (
                  SELECT 1 FROM outbox o
                  WHERE o.aggregate_type = 'power_plug'
                    AND o.aggregate_id = p.plug_id
                    AND o.created_at > NOW() - make_interval(secs => $2::double precision)
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
