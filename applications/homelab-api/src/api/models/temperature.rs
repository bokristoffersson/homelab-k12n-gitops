use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TemperatureReading {
    pub time: DateTime<Utc>,
    pub device_id: Option<String>,
    pub mac_address: Option<String>,
    pub location: Option<String>,
    pub temperature_c: Option<f64>,
    pub temperature_f: Option<f64>,
    pub humidity: Option<f64>,
    pub wifi_rssi: Option<f64>,
    pub battery_voltage: Option<f64>,
    pub battery_percent: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TemperatureLatest {
    pub time: DateTime<Utc>,
    pub location: Option<String>,
    pub temperature_c: Option<f64>,
    pub humidity: Option<f64>,
    pub battery_percent: Option<f64>,
}

impl From<crate::repositories::temperature::TemperatureReading> for TemperatureReading {
    fn from(reading: crate::repositories::temperature::TemperatureReading) -> Self {
        Self {
            time: reading.time,
            device_id: reading.device_id,
            mac_address: reading.mac_address,
            location: reading.location,
            temperature_c: reading.temperature_c,
            temperature_f: reading.temperature_f,
            humidity: reading.humidity,
            wifi_rssi: reading.wifi_rssi,
            battery_voltage: reading.battery_voltage,
            battery_percent: reading.battery_percent,
        }
    }
}

impl From<crate::repositories::temperature::TemperatureLatest> for TemperatureLatest {
    fn from(reading: crate::repositories::temperature::TemperatureLatest) -> Self {
        Self {
            time: reading.time,
            location: reading.location,
            temperature_c: reading.temperature_c,
            humidity: reading.humidity,
            battery_percent: reading.battery_percent,
        }
    }
}
