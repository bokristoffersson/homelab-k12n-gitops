use serde::{Deserialize, Serialize};

/// Heatpump settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeatpumpSetting {
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
    pub integral_setting: Option<f64>,
    pub updated_at: String,
}

/// Settings list response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsResponse {
    pub settings: Vec<HeatpumpSetting>,
}

/// Patch request for updating settings
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SettingPatch {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub indoor_target_temp: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub curve: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub curve_min: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub curve_max: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub curve_plus_5: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub curve_zero: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub curve_minus_5: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heatstop: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub integral_setting: Option<f64>,
}

/// Heatpump operation mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeatpumpMode {
    Off = 0,
    Heating = 1,
    Cooling = 2,
    Auto = 3,
}

impl HeatpumpMode {
    /// Get mode from integer value
    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::Off),
            1 => Some(Self::Heating),
            2 => Some(Self::Cooling),
            3 => Some(Self::Auto),
            _ => None,
        }
    }

    /// Get display name
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Off => "Off",
            Self::Heating => "Heating",
            Self::Cooling => "Cooling",
            Self::Auto => "Auto",
        }
    }
}
