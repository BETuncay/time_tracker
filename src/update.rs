use iced::Task;
use crate::model::{ActiveTimer, Model, ViewState, parse_hhmm_today};
use crate::db;
use std::time::Instant;

#[derive(Debug, Clone)]
pub enum Message {
    StartTask(String),
    StopCurrent,
    Tick(Instant),
    EditDescription(i64, String),
    DeleteEntry(i64),
    ShowView(ViewState),
    // manual entry form
    ManualFormTask(String),
    ManualFormDesc(String),
    ManualFormStart(String),
    ManualFormEnd(String),
    SubmitManualEntry,
}

impl Model {
    pub fn new() -> (Self, Task<Message>) {
        let conn = db::open().expect("failed to open database");
        let tasks = vec![
            "Development".to_string(),
            "Meetings".to_string(),
            "Review".to_string(),
            "Admin".to_string(),
        ];
        let entries = db::load_today(&conn).unwrap_or_default();
        let form_task = tasks.first().cloned().unwrap_or_default();
        let model = Self {
            tasks,
            active: None,
            entries,
            view_state: ViewState::Main,
            form_task,
            form_desc: String::new(),
            form_start: String::new(),
            form_end: String::new(),
            form_error: None,
        };
        (model, Task::none())
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::StartTask(name) => {
                // Stop current timer if running
                if self.active.is_some() {
                    self.stop_current();
                }
                let now = chrono::Utc::now().timestamp();
                self.active = Some(ActiveTimer {
                    task: name,
                    started_at: now,
                    elapsed_start: Instant::now(),
                });
                Task::none()
            }

            Message::StopCurrent => {
                self.stop_current();
                Task::none()
            }

            Message::Tick(_) => Task::none(),

            Message::EditDescription(id, text) => {
                if let Some(entry) = self.entries.iter_mut().find(|e| e.id == id) {
                    entry.description = text.clone();
                    let conn = db::open().expect("db open");
                    let _ = db::update_description(&conn, id, &text);
                }
                Task::none()
            }

            Message::DeleteEntry(id) => {
                let conn = db::open().expect("db open");
                let _ = db::delete_entry(&conn, id);
                self.entries.retain(|e| e.id != id);
                Task::none()
            }

            Message::ShowView(state) => {
                if matches!(state, ViewState::ManualEntry) {
                    // Reset form when opening
                    self.form_task = self.tasks.first().cloned().unwrap_or_default();
                    self.form_desc = String::new();
                    self.form_start = String::new();
                    self.form_end = String::new();
                    self.form_error = None;
                }
                self.view_state = state;
                Task::none()
            }

            Message::ManualFormTask(t) => { self.form_task = t; Task::none() }
            Message::ManualFormDesc(d) => { self.form_desc = d; Task::none() }
            Message::ManualFormStart(s) => { self.form_start = s; Task::none() }
            Message::ManualFormEnd(e) => { self.form_end = e; Task::none() }

            Message::SubmitManualEntry => {
                let start = parse_hhmm_today(&self.form_start);
                let end = parse_hhmm_today(&self.form_end);
                match (start, end) {
                    (Some(s), Some(e)) if e > s => {
                        let conn = db::open().expect("db open");
                        if let Ok(id) = db::insert_entry(&conn, &self.form_task, &self.form_desc, s, Some(e)) {
                            self.entries.push(crate::model::Entry {
                                id,
                                task: self.form_task.clone(),
                                description: self.form_desc.clone(),
                                started_at: s,
                                ended_at: Some(e),
                            });
                            self.entries.sort_by_key(|e| e.started_at);
                        }
                        self.view_state = ViewState::Main;
                        self.form_error = None;
                    }
                    (None, _) => self.form_error = Some("Invalid start time (use HH:MM)".into()),
                    (_, None) => self.form_error = Some("Invalid end time (use HH:MM)".into()),
                    _ => self.form_error = Some("End time must be after start time".into()),
                }
                Task::none()
            }
        }
    }

    fn stop_current(&mut self) {
        if let Some(timer) = self.active.take() {
            let ended_at = chrono::Utc::now().timestamp();
            let conn = db::open().expect("db open");
            if let Ok(id) = db::insert_entry(&conn, &timer.task, "", timer.started_at, Some(ended_at)) {
                self.entries.push(crate::model::Entry {
                    id,
                    task: timer.task,
                    description: String::new(),
                    started_at: timer.started_at,
                    ended_at: Some(ended_at),
                });
            }
        }
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        if self.active.is_some() {
            iced::time::every(std::time::Duration::from_secs(1))
                .map(Message::Tick)
        } else {
            iced::Subscription::none()
        }
    }
}
