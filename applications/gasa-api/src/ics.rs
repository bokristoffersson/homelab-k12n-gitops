use chrono::{DateTime, Utc};

use crate::models::Slot;

/// Build an RFC 5545 calendar for the shared family feed.
/// Free slots are included so available times show up on subscribed phones.
pub fn build_calendar(slots: &[Slot]) -> String {
    let mut lines: Vec<String> = vec![
        "BEGIN:VCALENDAR".into(),
        "VERSION:2.0".into(),
        "PRODID:-//k12n//gasa//SV".into(),
        "CALSCALE:GREGORIAN".into(),
        "METHOD:PUBLISH".into(),
        "X-WR-CALNAME:Övningskörning".into(),
        "X-WR-TIMEZONE:Europe/Stockholm".into(),
        "REFRESH-INTERVAL;VALUE=DURATION:PT15M".into(),
        "X-PUBLISHED-TTL:PT15M".into(),
    ];

    for slot in slots {
        let summary = match slot.display_name() {
            Some(name) => format!("Övningskörning - {}", name),
            None => "Övningskörning - Ledig".to_string(),
        };

        lines.push("BEGIN:VEVENT".into());
        // Stable UID + DTSTAMP from updated_at lets clients pick up book/cancel
        // changes on refresh instead of duplicating events.
        lines.push(format!("UID:gasa-slot-{}@k12n.com", slot.id));
        lines.push(format!("DTSTAMP:{}", format_utc(slot.updated_at)));
        lines.push(format!("DTSTART:{}", format_utc(slot.starts_at)));
        lines.push(format!("DTEND:{}", format_utc(slot.ends_at)));
        lines.push(format!("SUMMARY:{}", escape_text(&summary)));
        if let Some(note) = slot.note.as_deref().filter(|n| !n.is_empty()) {
            lines.push(format!("DESCRIPTION:{}", escape_text(note)));
        }
        lines.push("STATUS:CONFIRMED".into());
        lines.push("END:VEVENT".into());
    }

    lines.push("END:VCALENDAR".into());

    let mut out = String::new();
    for line in &lines {
        out.push_str(&fold_line(line));
        out.push_str("\r\n");
    }
    out
}

fn format_utc(dt: DateTime<Utc>) -> String {
    dt.format("%Y%m%dT%H%M%SZ").to_string()
}

/// Escape TEXT values per RFC 5545 section 3.3.11.
fn escape_text(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            ';' => out.push_str("\\;"),
            ',' => out.push_str("\\,"),
            '\n' => out.push_str("\\n"),
            '\r' => {}
            _ => out.push(ch),
        }
    }
    out
}

/// Fold content lines longer than 75 octets per RFC 5545 section 3.1.
/// Continuation lines begin with a single space (which counts toward their limit).
fn fold_line(line: &str) -> String {
    let mut out = String::with_capacity(line.len() + 8);
    let mut octets = 0usize;
    let mut first = true;
    for ch in line.chars() {
        let len = ch.len_utf8();
        let limit = if first { 75 } else { 74 };
        if octets + len > limit {
            out.push_str("\r\n ");
            first = false;
            octets = 0;
        }
        out.push(ch);
        octets += len;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn slot(id: i64, booked_by: Option<&str>, note: Option<&str>) -> Slot {
        Slot {
            id,
            starts_at: Utc.with_ymd_and_hms(2026, 8, 18, 17, 0, 0).unwrap(),
            ends_at: Utc.with_ymd_and_hms(2026, 8, 18, 18, 0, 0).unwrap(),
            note: note.map(str::to_owned),
            booked_by_username: booked_by.map(str::to_owned),
            booked_by_email: None,
            booked_at: None,
            updated_at: Utc.with_ymd_and_hms(2026, 8, 1, 12, 0, 0).unwrap(),
        }
    }

    #[test]
    fn test_empty_calendar_is_valid() {
        let cal = build_calendar(&[]);
        assert!(cal.starts_with("BEGIN:VCALENDAR\r\n"));
        assert!(cal.ends_with("END:VCALENDAR\r\n"));
        assert!(cal.contains("X-WR-CALNAME:Övningskörning\r\n"));
    }

    #[test]
    fn test_booked_and_free_summaries() {
        let cal = build_calendar(&[slot(1, Some("erik"), None), slot(2, None, None)]);
        assert!(cal.contains("SUMMARY:Övningskörning - Erik\r\n"));
        assert!(cal.contains("SUMMARY:Övningskörning - Ledig\r\n"));
    }

    #[test]
    fn test_stable_uid_and_utc_times() {
        let cal = build_calendar(&[slot(42, None, None)]);
        assert!(cal.contains("UID:gasa-slot-42@k12n.com\r\n"));
        assert!(cal.contains("DTSTART:20260818T170000Z\r\n"));
        assert!(cal.contains("DTEND:20260818T180000Z\r\n"));
        assert!(cal.contains("DTSTAMP:20260801T120000Z\r\n"));
    }

    #[test]
    fn test_note_is_escaped() {
        let cal = build_calendar(&[slot(1, None, Some("Motorväg; kör E4, sen\nhem"))]);
        assert!(cal.contains("DESCRIPTION:Motorväg\\; kör E4\\, sen\\nhem\r\n"));
    }

    #[test]
    fn test_long_lines_are_folded() {
        let long_note = "a".repeat(200);
        let cal = build_calendar(&[slot(1, None, Some(&long_note))]);
        for line in cal.split("\r\n") {
            assert!(line.len() <= 75, "line exceeds 75 octets: {}", line.len());
        }
        // Unfolding (removing CRLF + space) must restore the full note
        let unfolded = cal.replace("\r\n ", "");
        assert!(unfolded.contains(&format!("DESCRIPTION:{}", long_note)));
    }

    #[test]
    fn test_folding_respects_utf8_boundaries() {
        let long_note = "ö".repeat(100);
        let cal = build_calendar(&[slot(1, None, Some(&long_note))]);
        for line in cal.split("\r\n") {
            assert!(line.len() <= 75);
        }
        let unfolded = cal.replace("\r\n ", "");
        assert!(unfolded.contains(&long_note));
    }
}
