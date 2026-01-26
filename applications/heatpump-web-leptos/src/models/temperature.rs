use serde::{Deserialize, Serialize};

/// Full temperature reading from a sensor
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemperatureReading {
    pub time: String,
    pub device_id: Option<String>,
    pub mac_address: Option<String>,
    pub location: Option<String>,
    pub temperature_c: Option<f64>,
    pub temperature_f: Option<f64>,
    pub humidity: Option<f64>,
    pub wifi_rssi: Option<i32>,
    pub battery_voltage: Option<f64>,
    pub battery_percent: Option<f64>,
}

/// Latest temperature summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemperatureLatest {
    pub time: String,
    pub location: Option<String>,
    pub temperature_c: Option<f64>,
    pub humidity: Option<f64>,
    pub battery_percent: Option<f64>,
}
