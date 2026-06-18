//! Nord Pool day-ahead price client.
//!
//! Calls the same JSON API the public data portal uses. A browser-like
//! User-Agent is required - the endpoint returns 403 without one.

use crate::config::NordpoolConfig;
use crate::error::AppError;
use chrono::{DateTime, NaiveDate, Utc};
use serde::Deserialize;
use std::collections::HashMap;

/// One delivery period (typically one hour) of prices for the configured area.
#[derive(Debug, Clone)]
pub struct PriceEntry {
    pub start: DateTime<Utc>,
    #[allow(dead_code)]
    pub end: DateTime<Utc>,
    /// Price in the configured currency per MWh.
    pub price_per_mwh: f64,
}

/// Result of a day-ahead price fetch for one delivery date.
#[derive(Debug, Clone)]
pub struct DayAheadPrices {
    pub updated_at: Option<DateTime<Utc>>,
    pub currency: String,
    pub entries: Vec<PriceEntry>,
}

#[derive(Clone)]
pub struct NordpoolClient {
    http: reqwest::Client,
    base_url: String,
    area: String,
    currency: String,
}

impl NordpoolClient {
    pub fn new(cfg: &NordpoolConfig) -> Self {
        let http = reqwest::Client::builder()
            .user_agent(cfg.user_agent.clone())
            .build()
            .unwrap_or_default();
        Self {
            http,
            base_url: cfg.base_url.clone(),
            area: cfg.delivery_area.clone(),
            currency: cfg.currency.clone(),
        }
    }

    /// Fetch day-ahead prices for the given delivery date.
    /// Returns an empty `entries` list when prices are not yet published.
    pub async fn fetch(&self, date: NaiveDate) -> Result<DayAheadPrices, AppError> {
        let response = self
            .http
            .get(&self.base_url)
            .query(&[
                ("date", date.format("%Y-%m-%d").to_string()),
                ("market", "DayAhead".to_string()),
                ("deliveryArea", self.area.clone()),
                ("currency", self.currency.clone()),
            ])
            .send()
            .await?;

        // The API returns 204 (and sometimes 200 with an empty body) when the
        // requested day's prices have not been published yet.
        if response.status() == reqwest::StatusCode::NO_CONTENT {
            return Ok(DayAheadPrices {
                updated_at: None,
                currency: self.currency.clone(),
                entries: Vec::new(),
            });
        }

        let response = response.error_for_status()?;
        let body = response.text().await?;
        if body.trim().is_empty() {
            return Ok(DayAheadPrices {
                updated_at: None,
                currency: self.currency.clone(),
                entries: Vec::new(),
            });
        }

        let parsed: NordpoolResponse = serde_json::from_str(&body)?;
        let updated_at = parsed.updated_at.as_deref().and_then(parse_timestamp);

        let mut entries = Vec::with_capacity(parsed.multi_area_entries.len());
        for entry in parsed.multi_area_entries {
            if let Some(&price) = entry.entry_per_area.get(&self.area) {
                entries.push(PriceEntry {
                    start: entry.delivery_start,
                    end: entry.delivery_end,
                    price_per_mwh: price,
                });
            }
        }
        entries.sort_by_key(|e| e.start);

        Ok(DayAheadPrices {
            updated_at,
            currency: parsed.currency.unwrap_or_else(|| self.currency.clone()),
            entries,
        })
    }
}

/// Parse Nord Pool timestamps. They are RFC3339 but may carry sub-second
/// precision and sometimes omit a timezone designator (assumed UTC then).
fn parse_timestamp(raw: &str) -> Option<DateTime<Utc>> {
    if let Ok(dt) = DateTime::parse_from_rfc3339(raw) {
        return Some(dt.with_timezone(&Utc));
    }
    for fmt in ["%Y-%m-%dT%H:%M:%S%.f", "%Y-%m-%dT%H:%M:%S"] {
        if let Ok(naive) = chrono::NaiveDateTime::parse_from_str(raw, fmt) {
            return Some(DateTime::from_naive_utc_and_offset(naive, Utc));
        }
    }
    None
}

#[derive(Debug, Deserialize)]
struct NordpoolResponse {
    #[serde(default, rename = "updatedAt")]
    updated_at: Option<String>,
    #[serde(default, rename = "currency")]
    currency: Option<String>,
    #[serde(default, rename = "multiAreaEntries")]
    multi_area_entries: Vec<MultiAreaEntry>,
}

#[derive(Debug, Deserialize)]
struct MultiAreaEntry {
    #[serde(rename = "deliveryStart")]
    delivery_start: DateTime<Utc>,
    #[serde(rename = "deliveryEnd")]
    delivery_end: DateTime<Utc>,
    #[serde(rename = "entryPerArea")]
    entry_per_area: HashMap<String, f64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"{
        "deliveryDateCET": "2026-06-18",
        "updatedAt": "2026-06-17T11:00:34.1234567Z",
        "currency": "SEK",
        "deliveryAreas": ["SE3"],
        "multiAreaEntries": [
            {
                "deliveryStart": "2026-06-17T22:00:00Z",
                "deliveryEnd": "2026-06-17T23:00:00Z",
                "entryPerArea": { "SE3": 234.56 }
            },
            {
                "deliveryStart": "2026-06-17T23:00:00Z",
                "deliveryEnd": "2026-06-18T00:00:00Z",
                "entryPerArea": { "SE3": 198.10 }
            }
        ]
    }"#;

    #[test]
    fn parses_entries_for_area() {
        let parsed: NordpoolResponse = serde_json::from_str(SAMPLE).unwrap();
        let entries: Vec<_> = parsed
            .multi_area_entries
            .iter()
            .filter_map(|e| e.entry_per_area.get("SE3").map(|p| (e.delivery_start, *p)))
            .collect();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].1, 234.56);
    }

    #[test]
    fn parses_fractional_timestamp() {
        let ts = parse_timestamp("2026-06-17T11:00:34.1234567Z").unwrap();
        assert_eq!(ts.to_rfc3339().get(0..19), Some("2026-06-17T11:00:34"));
    }

    #[test]
    fn parses_timestamp_without_zone_as_utc() {
        let ts = parse_timestamp("2026-06-17T11:00:34").unwrap();
        assert_eq!(ts.to_rfc3339().get(0..19), Some("2026-06-17T11:00:34"));
    }
}
