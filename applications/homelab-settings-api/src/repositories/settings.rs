use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::error::{AppError, Result};

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct Setting {
    pub device_id: String,
    pub indoor_target_temp: Option<f64>,
    pub mode: Option<i32>,
    pub curve: Option<i32>,
    pub curve_min: Option<i32>,
    pub curve_max: Option<i32>,
    pub curve_plus_5: Option<i32>,
    pub curve_zero: Option<i32>,
    pub curve_minus_5: Option<i32>,
    pub heatstop: Option<i32>,
    pub integral_setting: Option<i16>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SettingUpdate {
    pub device_id: String,
    pub indoor_target_temp: Option<f64>,
    pub mode: Option<i32>,
    pub curve: Option<i32>,
    pub curve_min: Option<i32>,
    pub curve_max: Option<i32>,
    pub curve_plus_5: Option<i32>,
    pub curve_zero: Option<i32>,
    pub curve_minus_5: Option<i32>,
    pub heatstop: Option<i32>,
    pub integral_setting: Option<i16>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SettingPatch {
    pub indoor_target_temp: Option<f64>,
    pub mode: Option<i32>,
    pub curve: Option<i32>,
    pub curve_min: Option<i32>,
    pub curve_max: Option<i32>,
    pub curve_plus_5: Option<i32>,
    pub curve_zero: Option<i32>,
    pub curve_minus_5: Option<i32>,
    pub heatstop: Option<i32>,
    pub integral_setting: Option<i16>,
}

pub struct SettingsRepository {
    pool: PgPool,
}

impl SettingsRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Check database connectivity and table existence
    pub async fn health_check(&self) -> Result<(bool, bool)> {
        // Check connectivity
        sqlx::query("SELECT 1").execute(&self.pool).await?;

        // Check if settings table exists
        let exists: bool = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS (
                SELECT FROM information_schema.tables 
                WHERE table_schema = 'public' 
                AND table_name = 'settings'
            )",
        )
        .fetch_one(&self.pool)
        .await?;

        Ok((true, exists))
    }

    /// Get all device settings
    pub async fn get_all(&self) -> Result<Vec<Setting>> {
        let settings = sqlx::query_as::<_, Setting>(
            r#"
            SELECT device_id, indoor_target_temp, mode, curve, curve_min, curve_max,
                   curve_plus_5, curve_zero, curve_minus_5, heatstop, integral_setting, updated_at
            FROM settings
            ORDER BY device_id
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(settings)
    }

    /// Get settings for a specific device
    pub async fn get_by_device_id(&self, device_id: &str) -> Result<Setting> {
        let setting = sqlx::query_as::<_, Setting>(
            r#"
            SELECT device_id, indoor_target_temp, mode, curve, curve_min, curve_max,
                   curve_plus_5, curve_zero, curve_minus_5, heatstop, integral_setting, updated_at
            FROM settings
            WHERE device_id = $1
            "#,
        )
        .bind(device_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Device {} not found", device_id)))?;

        Ok(setting)
    }

    /// Upsert settings (used by Kafka consumer)
    ///
    /// Uses COALESCE strategy to prevent data loss during feature rollouts:
    /// - If field is present in Kafka message (non-NULL): Update to new value
    /// - If field is missing from Kafka message (NULL): Keep existing database value
    ///
    /// This protects against:
    /// - Partial messages from IoT devices during firmware updates
    /// - Corrupt messages missing previously-set fields
    /// - Gradual rollouts where not all devices send new fields yet
    ///
    /// This differs from the PATCH endpoint which uses dynamic SQL to only update
    /// fields explicitly provided in the request.
    pub async fn upsert(&self, update: &SettingUpdate) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO settings (
                device_id, indoor_target_temp, mode, curve, curve_min, curve_max,
                curve_plus_5, curve_zero, curve_minus_5, heatstop, integral_setting, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, NOW())
            ON CONFLICT (device_id) DO UPDATE SET
                indoor_target_temp = COALESCE(EXCLUDED.indoor_target_temp, settings.indoor_target_temp),
                mode = COALESCE(EXCLUDED.mode, settings.mode),
                curve = COALESCE(EXCLUDED.curve, settings.curve),
                curve_min = COALESCE(EXCLUDED.curve_min, settings.curve_min),
                curve_max = COALESCE(EXCLUDED.curve_max, settings.curve_max),
                curve_plus_5 = COALESCE(EXCLUDED.curve_plus_5, settings.curve_plus_5),
                curve_zero = COALESCE(EXCLUDED.curve_zero, settings.curve_zero),
                curve_minus_5 = COALESCE(EXCLUDED.curve_minus_5, settings.curve_minus_5),
                heatstop = COALESCE(EXCLUDED.heatstop, settings.heatstop),
                integral_setting = COALESCE(EXCLUDED.integral_setting, settings.integral_setting),
                updated_at = NOW()
            "#,
        )
        .bind(&update.device_id)
        .bind(update.indoor_target_temp)
        .bind(update.mode)
        .bind(update.curve)
        .bind(update.curve_min)
        .bind(update.curve_max)
        .bind(update.curve_plus_5)
        .bind(update.curve_zero)
        .bind(update.curve_minus_5)
        .bind(update.heatstop)
        .bind(update.integral_setting)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Partially update settings (PATCH endpoint)
    /// Note: This method is replaced by update_setting_in_tx for transactional outbox pattern
    #[allow(dead_code)]
    pub async fn update(&self, device_id: &str, patch: &SettingPatch) -> Result<Setting> {
        // Build dynamic UPDATE query based on which fields are present
        let mut query = String::from("UPDATE settings SET updated_at = NOW()");
        let mut bind_count = 1;

        if patch.indoor_target_temp.is_some() {
            bind_count += 1;
            query.push_str(&format!(", indoor_target_temp = ${}", bind_count));
        }
        if patch.mode.is_some() {
            bind_count += 1;
            query.push_str(&format!(", mode = ${}", bind_count));
        }
        if patch.curve.is_some() {
            bind_count += 1;
            query.push_str(&format!(", curve = ${}", bind_count));
        }
        if patch.curve_min.is_some() {
            bind_count += 1;
            query.push_str(&format!(", curve_min = ${}", bind_count));
        }
        if patch.curve_max.is_some() {
            bind_count += 1;
            query.push_str(&format!(", curve_max = ${}", bind_count));
        }
        if patch.curve_plus_5.is_some() {
            bind_count += 1;
            query.push_str(&format!(", curve_plus_5 = ${}", bind_count));
        }
        if patch.curve_zero.is_some() {
            bind_count += 1;
            query.push_str(&format!(", curve_zero = ${}", bind_count));
        }
        if patch.curve_minus_5.is_some() {
            bind_count += 1;
            query.push_str(&format!(", curve_minus_5 = ${}", bind_count));
        }
        if patch.heatstop.is_some() {
            bind_count += 1;
            query.push_str(&format!(", heatstop = ${}", bind_count));
        }
        if patch.integral_setting.is_some() {
            bind_count += 1;
            query.push_str(&format!(", integral_setting = ${}", bind_count));
        }

        query.push_str(" WHERE device_id = $1 RETURNING *");

        let mut query_builder = sqlx::query_as::<_, Setting>(&query).bind(device_id);

        if let Some(val) = patch.indoor_target_temp {
            query_builder = query_builder.bind(val);
        }
        if let Some(val) = patch.mode {
            query_builder = query_builder.bind(val);
        }
        if let Some(val) = patch.curve {
            query_builder = query_builder.bind(val);
        }
        if let Some(val) = patch.curve_min {
            query_builder = query_builder.bind(val);
        }
        if let Some(val) = patch.curve_max {
            query_builder = query_builder.bind(val);
        }
        if let Some(val) = patch.curve_plus_5 {
            query_builder = query_builder.bind(val);
        }
        if let Some(val) = patch.curve_zero {
            query_builder = query_builder.bind(val);
        }
        if let Some(val) = patch.curve_minus_5 {
            query_builder = query_builder.bind(val);
        }
        if let Some(val) = patch.heatstop {
            query_builder = query_builder.bind(val);
        }
        if let Some(val) = patch.integral_setting {
            query_builder = query_builder.bind(val);
        }

        let setting = query_builder
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Device {} not found", device_id)))?;

        Ok(setting)
    }
}
