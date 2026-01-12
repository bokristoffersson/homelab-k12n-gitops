use chrono::{DateTime, Utc};
use serde::Serialize;

pub use crate::repositories::settings::Setting;

#[derive(Debug, Serialize)]
pub struct SettingsListResponse {
    pub settings: Vec<Setting>,
}

#[derive(Debug, Serialize)]
pub struct SettingResponse {
    #[serde(flatten)]
    pub setting: Setting,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outbox_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outbox_status: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct OutboxStatusResponse {
    pub id: i64,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub published_at: Option<DateTime<Utc>>,
    pub confirmed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
    pub retry_count: i32,
}
