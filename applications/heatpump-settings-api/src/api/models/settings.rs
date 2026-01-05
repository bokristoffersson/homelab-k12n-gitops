use serde::{Deserialize, Serialize};

pub use crate::repositories::settings::{Setting, SettingPatch};

#[derive(Debug, Serialize)]
pub struct SettingsListResponse {
    pub settings: Vec<Setting>,
}

#[derive(Debug, Serialize)]
pub struct SettingResponse {
    #[serde(flatten)]
    pub setting: Setting,
}
