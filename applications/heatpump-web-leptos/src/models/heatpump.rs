use serde::{Deserialize, Serialize};

/// Heatpump status from the monitoring system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeatpumpStatus {
    pub ts: String,
    pub device_id: Option<String>,
    pub compressor_on: Option<bool>,
    pub hotwater_production: Option<bool>,
    pub flowlinepump_on: Option<bool>,
    pub brinepump_on: Option<bool>,
    pub aux_heater_3kw_on: Option<bool>,
    pub aux_heater_6kw_on: Option<bool>,
    pub outdoor_temp: Option<f64>,
    pub supplyline_temp: Option<f64>,
    pub returnline_temp: Option<f64>,
    pub hotwater_temp: Option<f64>,
    pub brine_out_temp: Option<f64>,
    pub brine_in_temp: Option<f64>,
    pub integral: Option<f64>,
}
