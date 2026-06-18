//! Daily Nord Pool fetch scheduler.
//!
//! Once per day, at a randomized time inside the configured window, fetches
//! tomorrow's SE3 day-ahead prices, stores them, and pushes a notification
//! with the day's lowest/highest price. Retries within the window if prices
//! are not yet published. Also bootstraps today's prices on startup.

use crate::apns::ApnsSender;
use crate::config::Config;
use crate::db::DbPool;
use crate::nordpool::{NordpoolClient, PriceEntry};
use crate::repositories::{DeviceTokenRepository, SpotPriceRepository};
use chrono::{DateTime, Local, NaiveDate, NaiveTime, TimeZone};
use rand::Rng;
use std::time::Duration;
use tokio::time::interval;

/// Result of a fetch+store attempt for one delivery date.
enum FetchOutcome {
    /// Nord Pool has not published prices for the date yet.
    NotPublished,
    /// Prices are stored. `is_new` is true only the first time the date's
    /// prices land in the DB, so the push fires exactly once per publication
    /// (idempotent across pod restarts — survives the in-memory day flag reset).
    Stored { summary: PriceSummary, is_new: bool },
}

/// Lowest/highest price summary used to build the push notification.
struct PriceSummary {
    date: NaiveDate,
    currency: String,
    min_ore: f64,
    min_time: DateTime<Local>,
    max_ore: f64,
    max_time: DateTime<Local>,
}

pub struct Fetcher {
    pool: DbPool,
    config: Config,
    apns: Option<ApnsSender>,
    nordpool: NordpoolClient,
}

impl Fetcher {
    pub fn new(pool: DbPool, config: Config, apns: Option<ApnsSender>) -> Self {
        let nordpool = NordpoolClient::new(&config.nordpool);
        Self {
            pool,
            config,
            apns,
            nordpool,
        }
    }

    pub async fn run(self) {
        let fetch = &self.config.fetch;
        let window_start = parse_time(
            &fetch.window_start,
            NaiveTime::from_hms_opt(13, 30, 0).unwrap(),
        );
        let window_end = parse_time(
            &fetch.window_end,
            NaiveTime::from_hms_opt(14, 0, 0).unwrap(),
        );
        let retry_until = parse_time(
            &fetch.retry_until,
            NaiveTime::from_hms_opt(16, 0, 0).unwrap(),
        );
        let retry_interval = chrono::Duration::seconds(fetch.retry_interval_secs as i64);

        tracing::info!(
            "Fetcher started: window {}-{}, retry until {}, area {}",
            window_start.format("%H:%M"),
            window_end.format("%H:%M"),
            retry_until.format("%H:%M"),
            self.config.nordpool.delivery_area
        );

        // Bootstrap today's prices so the /today endpoint works immediately.
        if let Err(e) = self.ensure_today_prices().await {
            tracing::warn!("Bootstrap of today's prices failed: {}", e);
        }

        let mut day: Option<NaiveDate> = None;
        let mut target: NaiveTime = window_start;
        let mut tomorrow_done = false;
        let mut last_attempt: Option<DateTime<Local>> = None;

        let mut ticker = interval(Duration::from_secs(fetch.check_interval_secs.max(1)));
        loop {
            ticker.tick().await;
            let now = Local::now();
            let today = now.date_naive();

            if day != Some(today) {
                day = Some(today);
                target = pick_target(window_start, window_end);
                tomorrow_done = false;
                last_attempt = None;
                tracing::info!(
                    "New day {}: tomorrow fetch target {}",
                    today,
                    target.format("%H:%M")
                );
                if let Err(e) = self.ensure_today_prices().await {
                    tracing::warn!("ensure_today_prices failed: {}", e);
                }
            }

            if tomorrow_done {
                continue;
            }

            let now_time = now.time();
            if now_time < target || now_time > retry_until {
                continue;
            }
            if let Some(la) = last_attempt {
                if now.signed_duration_since(la) < retry_interval {
                    continue;
                }
            }
            last_attempt = Some(now);

            let tomorrow = match today.succ_opt() {
                Some(d) => d,
                None => continue,
            };
            match self.fetch_and_store(tomorrow).await {
                Ok(FetchOutcome::Stored { summary, is_new }) => {
                    tomorrow_done = true;
                    if is_new {
                        self.notify(&summary).await;
                    } else {
                        tracing::info!("Tomorrow's prices already stored; skipping duplicate push");
                    }
                }
                Ok(FetchOutcome::NotPublished) => {
                    tracing::info!("Tomorrow's prices not published yet, will retry");
                }
                Err(e) => {
                    tracing::warn!("Fetch of tomorrow's prices failed: {}", e);
                }
            }
        }
    }

    /// Ensure today's prices are present; fetch them once if missing. No push.
    async fn ensure_today_prices(&self) -> anyhow::Result<()> {
        let today = Local::now().date_naive();
        let count = SpotPriceRepository::count_for_local_date(
            &self.pool,
            &self.config.nordpool.delivery_area,
            today,
        )
        .await?;
        if count > 0 {
            return Ok(());
        }
        tracing::info!("No prices for today ({}), fetching for bootstrap", today);
        self.fetch_and_store(today).await?;
        Ok(())
    }

    /// Fetch and store one delivery date. Reports whether the date's prices
    /// were newly populated (used to gate the push notification).
    async fn fetch_and_store(&self, date: NaiveDate) -> anyhow::Result<FetchOutcome> {
        let area = &self.config.nordpool.delivery_area;
        let data = self.nordpool.fetch(date).await?;
        if data.entries.is_empty() {
            return Ok(FetchOutcome::NotPublished);
        }

        // Was the date already stored before this run? Push only on first arrival.
        let existing = SpotPriceRepository::count_for_local_date(&self.pool, area, date).await?;
        let is_new = existing == 0;

        SpotPriceRepository::upsert_day(
            &self.pool,
            area,
            &data.currency,
            &data.entries,
            data.updated_at,
            chrono::Utc::now(),
        )
        .await?;

        tracing::info!(
            "Stored {} price periods for {} ({})",
            data.entries.len(),
            date,
            area
        );

        match summarize(date, &data.currency, &data.entries) {
            Some(summary) => Ok(FetchOutcome::Stored { summary, is_new }),
            None => Ok(FetchOutcome::NotPublished),
        }
    }

    /// Push the day's lowest/highest price to all registered devices.
    async fn notify(&self, summary: &PriceSummary) {
        let Some(apns) = self.apns.as_ref() else {
            tracing::info!("APNs not configured; skipping push for {}", summary.date);
            return;
        };

        let tokens = match DeviceTokenRepository::all(&self.pool).await {
            Ok(t) => t,
            Err(e) => {
                tracing::warn!("Failed to load device tokens: {}", e);
                return;
            }
        };
        if tokens.is_empty() {
            tracing::info!("No registered devices; skipping push for {}", summary.date);
            return;
        }

        let title = "Elpriser imorgon (SE3)";
        let body = format!(
            "Lägst {:.0} öre kl {}, högst {:.0} öre kl {}",
            summary.min_ore,
            summary.min_time.format("%H:%M"),
            summary.max_ore,
            summary.max_time.format("%H:%M")
        );
        let custom = serde_json::json!({
            "date": summary.date.format("%Y-%m-%d").to_string(),
            "currency": summary.currency,
            "min": { "ore_per_kwh": summary.min_ore, "time": summary.min_time.to_rfc3339() },
            "max": { "ore_per_kwh": summary.max_ore, "time": summary.max_time.to_rfc3339() },
        });

        let mut sent = 0usize;
        for device in &tokens {
            match apns
                .send(&device.token, &device.environment, title, &body, &custom)
                .await
            {
                Ok(()) => sent += 1,
                Err(e) => tracing::warn!("APNs send failed for a device: {}", e),
            }
        }
        tracing::info!(
            "Pushed price notification to {}/{} devices",
            sent,
            tokens.len()
        );
    }
}

/// Build a min/max summary from the day's entries. Öre/kWh = (SEK/MWh) / 10.
fn summarize(date: NaiveDate, currency: &str, entries: &[PriceEntry]) -> Option<PriceSummary> {
    let min = entries
        .iter()
        .min_by(|a, b| a.price_per_mwh.total_cmp(&b.price_per_mwh))?;
    let max = entries
        .iter()
        .max_by(|a, b| a.price_per_mwh.total_cmp(&b.price_per_mwh))?;
    Some(PriceSummary {
        date,
        currency: currency.to_string(),
        min_ore: min.price_per_mwh / 10.0,
        min_time: min.start.with_timezone(&Local),
        max_ore: max.price_per_mwh / 10.0,
        max_time: max.start.with_timezone(&Local),
    })
}

fn parse_time(raw: &str, fallback: NaiveTime) -> NaiveTime {
    NaiveTime::parse_from_str(raw, "%H:%M").unwrap_or(fallback)
}

/// Pick a random target time in [start, end). Falls back to `start` if the
/// window is empty or inverted.
fn pick_target(start: NaiveTime, end: NaiveTime) -> NaiveTime {
    let span = (end - start).num_seconds();
    if span <= 0 {
        return start;
    }
    let offset = rand::thread_rng().gen_range(0..span);
    start + chrono::Duration::seconds(offset)
}

/// Convenience used by tests / callers needing a Local timestamp from a date.
#[allow(dead_code)]
fn local_midnight(date: NaiveDate) -> Option<DateTime<Local>> {
    Local
        .from_local_datetime(&date.and_hms_opt(0, 0, 0)?)
        .single()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};

    fn entry(start_h: u32, price: f64) -> PriceEntry {
        let start = Utc.with_ymd_and_hms(2026, 6, 18, start_h, 0, 0).unwrap();
        PriceEntry {
            start,
            end: start + chrono::Duration::hours(1),
            price_per_mwh: price,
        }
    }

    #[test]
    fn summarize_picks_min_and_max() {
        let date = NaiveDate::from_ymd_opt(2026, 6, 18).unwrap();
        let entries = vec![entry(0, 500.0), entry(1, 100.0), entry(2, 1870.0)];
        let s = summarize(date, "SEK", &entries).unwrap();
        // 100 SEK/MWh -> 10 öre/kWh, 1870 -> 187 öre/kWh
        assert_eq!(s.min_ore, 10.0);
        assert_eq!(s.max_ore, 187.0);
    }

    #[test]
    fn pick_target_within_window() {
        let start = NaiveTime::from_hms_opt(13, 30, 0).unwrap();
        let end = NaiveTime::from_hms_opt(14, 0, 0).unwrap();
        for _ in 0..100 {
            let t = pick_target(start, end);
            assert!(t >= start && t < end);
        }
    }

    #[test]
    fn pick_target_handles_empty_window() {
        let t = NaiveTime::from_hms_opt(13, 30, 0).unwrap();
        assert_eq!(pick_target(t, t), t);
    }

    #[test]
    fn parse_time_falls_back_on_garbage() {
        let fb = NaiveTime::from_hms_opt(14, 0, 0).unwrap();
        assert_eq!(parse_time("not-a-time", fb), fb);
        assert_eq!(
            parse_time("13:30", fb),
            NaiveTime::from_hms_opt(13, 30, 0).unwrap()
        );
    }
}
