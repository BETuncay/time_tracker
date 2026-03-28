use std::time::Instant;

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

#[derive(Debug, Clone)]
pub struct Model {
    pub tasks: Vec<String>,
    pub active: Option<ActiveTimer>,
    pub entries: Vec<Entry>,
    pub view_state: ViewState,
}
