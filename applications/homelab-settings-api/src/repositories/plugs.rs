use chrono::{DateTime, NaiveTime, Timelike, Utc};
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

    /// Update plug status within a transaction (for outbox pattern)
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
            SELECT id, plug_id, action, time_of_day, enabled, created_at, updated_at
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
            SELECT id, plug_id, action, time_of_day, enabled, created_at, updated_at
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
            RETURNING id, plug_id, action, time_of_day, enabled, created_at, updated_at
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

        query.push_str(" WHERE id = $1 RETURNING id, plug_id, action, time_of_day, enabled, created_at, updated_at");

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

    /// Get enabled schedules due in the current minute (for scheduler in Phase 6)
    #[allow(dead_code)]
    pub async fn get_due_schedules(
        &self,
        current_time: NaiveTime,
    ) -> Result<Vec<PowerPlugSchedule>> {
        // Match schedules within the current minute (00 seconds to 59 seconds)
        let time_start = current_time.with_second(0).unwrap_or(current_time);
        let time_end = current_time.with_second(59).unwrap_or(current_time);

        let schedules = sqlx::query_as::<_, PowerPlugSchedule>(
            r#"
            SELECT id, plug_id, action, time_of_day, enabled, created_at, updated_at
            FROM power_plug_schedules
            WHERE enabled = true
              AND time_of_day >= $1
              AND time_of_day <= $2
            "#,
        )
        .bind(time_start)
        .bind(time_end)
        .fetch_all(&self.pool)
        .await?;

        Ok(schedules)
    }
}
