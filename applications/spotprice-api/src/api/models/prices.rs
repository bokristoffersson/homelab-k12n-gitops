use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct PricePoint {
    pub time: DateTime<Utc>,
    pub price_per_kwh: f64,
    pub price_ore_per_kwh: f64,
}

#[derive(Debug, Serialize)]
pub struct PriceExtreme {
    pub time: DateTime<Utc>,
    pub price_ore_per_kwh: f64,
}

#[derive(Debug, Serialize)]
pub struct PricesResponse {
    pub delivery_area: String,
    pub currency: String,
    pub date: NaiveDate,
    pub source_updated_at: Option<DateTime<Utc>>,
    pub min: Option<PriceExtreme>,
    pub max: Option<PriceExtreme>,
    pub prices: Vec<PricePoint>,
}

#[derive(Debug, Serialize)]
pub struct LatestResponse {
    pub today: PricesResponse,
    pub tomorrow: Option<PricesResponse>,
}
