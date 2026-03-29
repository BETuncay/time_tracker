use std::time::Instant;

/// Format a duration in seconds as "HH:MM:SS".
pub fn format_duration(secs: i64) -> String {
    let secs = secs.max(0) as u64;
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    let s = secs % 60;
    format!("{:02}:{:02}:{:02}", h, m, s)
}

/// Format a unix timestamp as local time "HH:MM".
pub fn format_time(ts: i64) -> String {
    use chrono::{Local, TimeZone};
    Local
        .timestamp_opt(ts, 0)
        .single()
        .map(|dt| dt.format("%H:%M").to_string())
        .unwrap_or_else(|| "--:--".to_string())
}

/// Format a duration in seconds as "HHh MMm".
pub fn format_hm(secs: i64) -> String {
    let secs = secs.max(0) as u64;
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    format!("{:02}h {:02}m", h, m)
}

#[derive(Debug, Clone)]
pub struct Entry {
    pub id: i64,
    pub task: String,
    pub description: String,
    pub started_at: i64,  // unix timestamp
    pub ended_at: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct ActiveTimer {
    pub task: String,
    pub started_at: i64,
    pub elapsed_start: Instant,
}

#[derive(Debug, Clone)]
pub enum ViewState {
    Main,
    ManualEntry,
    EditEntry(i64),
    TaskManagement,
    Report,
}

/// Parse a "HH:MM" string against today's local date, returning a UTC unix timestamp.
/// Returns None if the string is not a valid time.
pub fn parse_hhmm_today(s: &str) -> Option<i64> {
    use chrono::{Local, NaiveTime, TimeZone};
    let t = NaiveTime::parse_from_str(s.trim(), "%H:%M").ok()?;
    let today = Local::now().date_naive();
    Local
        .from_local_datetime(&today.and_time(t))
        .single()
        .map(|dt| dt.timestamp())
}

#[derive(Debug, Clone)]
pub struct Model {
    pub tasks: Vec<String>,
    pub active: Option<ActiveTimer>,
    pub entries: Vec<Entry>,
    pub view_state: ViewState,
    // manual entry form state
    pub form_task: String,
    pub form_desc: String,
    pub form_start: String,
    pub form_end: String,
    pub form_error: Option<String>,
}

impl Entry {
    /// Duration in seconds, or 0 if the entry has no end time.
    pub fn duration_secs(&self) -> i64 {
        self.ended_at.map(|e| e - self.started_at).unwrap_or(0).max(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_duration_zero() {
        assert_eq!(format_duration(0), "00:00:00");
    }

    #[test]
    fn format_duration_one_hour() {
        assert_eq!(format_duration(3661), "01:01:01");
    }

    #[test]
    fn format_duration_negative_clamps_to_zero() {
        assert_eq!(format_duration(-5), "00:00:00");
    }

    #[test]
    fn format_hm_basic() {
        assert_eq!(format_hm(3661), "01h 01m");
    }

    #[test]
    fn format_time_invalid_returns_placeholder() {
        // i64::MIN is not a valid timestamp; should return the fallback
        assert_eq!(format_time(i64::MIN), "--:--");
    }

    #[test]
    fn entry_duration_secs_closed() {
        let e = Entry {
            id: 1,
            task: "Dev".into(),
            description: "".into(),
            started_at: 1000,
            ended_at: Some(4600),
        };
        assert_eq!(e.duration_secs(), 3600);
    }

    #[test]
    fn parse_hhmm_today_valid() {
        assert!(parse_hhmm_today("09:30").is_some());
        assert!(parse_hhmm_today("00:00").is_some());
        assert!(parse_hhmm_today("23:59").is_some());
    }

    #[test]
    fn parse_hhmm_today_invalid() {
        assert!(parse_hhmm_today("").is_none());
        assert!(parse_hhmm_today("25:00").is_none());
        assert!(parse_hhmm_today("abc").is_none());
        assert!(parse_hhmm_today("9:300").is_none());
    }

    #[test]
    fn entry_duration_secs_open() {
        let e = Entry {
            id: 2,
            task: "Dev".into(),
            description: "".into(),
            started_at: 1000,
            ended_at: None,
        };
        assert_eq!(e.duration_secs(), 0);
    }
}
