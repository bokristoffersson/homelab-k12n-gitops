use serde::{Deserialize, Serialize};

/// Latest energy reading from the power meter
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnergyLatest {
    pub ts: String,
    pub consumption_total_w: Option<f64>,
    pub consumption_total_actual_w: Option<f64>,
    pub consumption_l1_w: Option<f64>,
    pub consumption_l2_w: Option<f64>,
    pub consumption_l3_w: Option<f64>,
}

/// Hourly energy total
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HourlyTotal {
    pub total_kwh: f64,
    pub hour_start: String,
    pub current_time: String,
}

/// Hourly energy breakdown
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnergyHourly {
    pub hour_start: String,
    pub hour_end: String,
    pub total_energy_kwh: Option<f64>,
    pub total_energy_l1_kwh: Option<f64>,
    pub total_energy_l2_kwh: Option<f64>,
    pub total_energy_l3_kwh: Option<f64>,
    pub total_energy_actual_kwh: Option<f64>,
    pub measurement_count: i32,
}
