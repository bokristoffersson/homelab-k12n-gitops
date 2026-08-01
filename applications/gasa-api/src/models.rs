use chrono::{DateTime, Datelike, Timelike, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Slot {
    pub id: i64,
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
    pub note: Option<String>,
    pub booked_by_username: Option<String>,
    pub booked_by_email: Option<String>,
    pub booked_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

impl Slot {
    pub fn display_name(&self) -> Option<String> {
        self.booked_by_username.as_deref().map(display_name)
    }
}

/// Derive a display name from a username: first letter uppercased.
pub fn display_name(username: &str) -> String {
    let mut chars = username.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

/// Format a slot's time range in Swedish local time, e.g. "tis 18 aug 19:00-20:00".
/// Display formatting only - all storage and API payloads stay in UTC.
pub fn format_range_sv(starts_at: DateTime<Utc>, ends_at: DateTime<Utc>) -> String {
    const WEEKDAYS: [&str; 7] = ["mån", "tis", "ons", "tor", "fre", "lör", "sön"];
    const MONTHS: [&str; 12] = [
        "jan", "feb", "mar", "apr", "maj", "jun", "jul", "aug", "sep", "okt", "nov", "dec",
    ];

    let s = starts_at.with_timezone(&chrono_tz::Europe::Stockholm);
    let e = ends_at.with_timezone(&chrono_tz::Europe::Stockholm);
    format!(
        "{} {} {} {:02}:{:02}-{:02}:{:02}",
        WEEKDAYS[s.weekday().num_days_from_monday() as usize],
        s.day(),
        MONTHS[s.month0() as usize],
        s.hour(),
        s.minute(),
        e.hour(),
        e.minute()
    )
}

#[derive(Debug, Serialize)]
pub struct BookedBy {
    pub username: String,
    pub display_name: String,
}

#[derive(Debug, Serialize)]
pub struct SlotResponse {
    pub id: i64,
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
    pub note: Option<String>,
    pub booked_by: Option<BookedBy>,
    pub booked_at: Option<DateTime<Utc>>,
}

impl From<Slot> for SlotResponse {
    fn from(slot: Slot) -> Self {
        let booked_by = slot.booked_by_username.as_deref().map(|u| BookedBy {
            username: u.to_string(),
            display_name: display_name(u),
        });
        SlotResponse {
            id: slot.id,
            starts_at: slot.starts_at,
            ends_at: slot.ends_at,
            note: slot.note,
            booked_by,
            booked_at: slot.booked_at,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateSlotRequest {
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
    #[serde(default)]
    pub note: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct MeResponse {
    pub username: String,
    pub email: Option<String>,
    pub is_admin: bool,
}

#[derive(Debug, Serialize)]
pub struct CalendarUrlResponse {
    pub webcal_url: String,
    pub https_url: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_display_name_capitalizes() {
        assert_eq!(display_name("erik"), "Erik");
        assert_eq!(display_name("Erik"), "Erik");
        assert_eq!(display_name("åsa"), "Åsa");
        assert_eq!(display_name(""), "");
    }

    #[test]
    fn test_format_range_sv_summer_time() {
        // 2026-08-18 17:00 UTC = 19:00 CEST (Tuesday)
        let starts = Utc.with_ymd_and_hms(2026, 8, 18, 17, 0, 0).unwrap();
        let ends = Utc.with_ymd_and_hms(2026, 8, 18, 18, 0, 0).unwrap();
        assert_eq!(format_range_sv(starts, ends), "tis 18 aug 19:00-20:00");
    }

    #[test]
    fn test_format_range_sv_winter_time() {
        // 2026-12-07 18:00 UTC = 19:00 CET (Monday)
        let starts = Utc.with_ymd_and_hms(2026, 12, 7, 18, 0, 0).unwrap();
        let ends = Utc.with_ymd_and_hms(2026, 12, 7, 19, 30, 0).unwrap();
        assert_eq!(format_range_sv(starts, ends), "mån 7 dec 19:00-20:30");
    }
}
