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
}
