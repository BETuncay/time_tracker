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

/// Returns `(task, today_secs, week_secs)` sorted by task name.
/// `today_start` is the unix timestamp of local midnight today.
pub fn compute_report_totals(entries: &[Entry], today_start: i64) -> Vec<(String, i64, i64)> {
    use std::collections::HashMap;
    let mut totals: HashMap<String, (i64, i64)> = HashMap::new();
    for entry in entries {
        let dur = entry.duration_secs();
        let bucket = totals.entry(entry.task.clone()).or_insert((0, 0));
        bucket.1 += dur;
        if entry.started_at >= today_start {
            bucket.0 += dur;
        }
    }
    let mut result: Vec<(String, i64, i64)> = totals
        .into_iter()
        .map(|(task, (today, week))| (task, today, week))
        .collect();
    result.sort_by(|a, b| a.0.cmp(&b.0));
    result
}

#[derive(Debug, Clone)]
pub struct Model {
    pub tasks: Vec<String>,
    pub active: Option<ActiveTimer>,
    pub entries: Vec<Entry>,
    pub report_entries: Vec<Entry>,
    pub view_state: ViewState,
    // manual entry form state
    pub form_task: String,
    pub form_desc: String,
    pub form_start: String,
    pub form_end: String,
    pub form_error: Option<String>,
    // task management state
    pub task_new_name: String,
    pub task_new_error: Option<String>,
    pub task_renaming: Option<usize>,  // index of task being renamed
    pub task_rename_text: String,
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

    #[test]
    fn compute_report_totals_basic() {
        let entries = vec![
            Entry { id: 1, task: "Dev".into(), description: "".into(), started_at: 1000, ended_at: Some(4600) }, // 3600s
            Entry { id: 2, task: "Dev".into(), description: "".into(), started_at: 5000, ended_at: Some(6800) }, // 1800s
            Entry { id: 3, task: "Meetings".into(), description: "".into(), started_at: 500, ended_at: Some(2300) }, // 1800s
        ];
        // today_start = 900: entries 1 and 2 are today, entry 3 is earlier this week
        let totals = compute_report_totals(&entries, 900);
        assert_eq!(totals.len(), 2);
        // sorted: Dev, Meetings
        let dev = totals.iter().find(|(t, _, _)| t == "Dev").unwrap();
        assert_eq!(dev.1, 5400); // today: 3600+1800
        assert_eq!(dev.2, 5400); // week: same
        let meetings = totals.iter().find(|(t, _, _)| t == "Meetings").unwrap();
        assert_eq!(meetings.1, 0);    // not today
        assert_eq!(meetings.2, 1800); // week
    }

    #[test]
    fn compute_report_totals_empty() {
        let totals = compute_report_totals(&[], 0);
        assert!(totals.is_empty());
    }

    #[test]
    fn compute_report_totals_sorted_by_name() {
        let entries = vec![
            Entry { id: 1, task: "Zzz".into(), description: "".into(), started_at: 100, ended_at: Some(200) },
            Entry { id: 2, task: "Aaa".into(), description: "".into(), started_at: 100, ended_at: Some(200) },
        ];
        let totals = compute_report_totals(&entries, 0);
        assert_eq!(totals[0].0, "Aaa");
        assert_eq!(totals[1].0, "Zzz");
    }

    // Task management invariants
    #[test]
    fn task_duplicate_detection_is_case_insensitive() {
        let tasks = vec!["Development".to_string(), "Meetings".to_string()];
        // Same case — duplicate
        assert!(tasks.iter().any(|t| t.eq_ignore_ascii_case("Development")));
        // Different case — still duplicate
        assert!(tasks.iter().any(|t| t.eq_ignore_ascii_case("development")));
        assert!(tasks.iter().any(|t| t.eq_ignore_ascii_case("MEETINGS")));
        // Not a duplicate
        assert!(!tasks.iter().any(|t| t.eq_ignore_ascii_case("Admin")));
    }

    #[test]
    fn task_rename_excludes_self_from_duplicate_check() {
        let tasks = vec!["Dev".to_string(), "Meetings".to_string()];
        let idx = 0;
        let new_name = "dev"; // same task, just lower-case — should be allowed
        let duplicate = tasks.iter().enumerate()
            .any(|(i, t)| i != idx && t.eq_ignore_ascii_case(new_name));
        assert!(!duplicate, "renaming a task to a case variation of itself should not be blocked");
    }
}
