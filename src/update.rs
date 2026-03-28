use iced::Task;
use crate::model::{ActiveTimer, Model, ViewState};
use crate::db;
use std::time::Instant;

#[derive(Debug, Clone)]
pub enum Message {
    StartTask(String),
    StopCurrent,
    Tick(Instant),
    EditDescription(i64, String),
    AddManualEntry {
        task: String,
        description: String,
        start: i64,
        end: i64,
    },
    DeleteEntry(i64),
    ShowView(ViewState),
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
        let model = Self {
            tasks,
            active: None,
            entries,
            view_state: ViewState::Main,
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

            Message::AddManualEntry { task, description, start, end } => {
                let conn = db::open().expect("db open");
                if let Ok(id) = db::insert_entry(&conn, &task, &description, start, Some(end)) {
                    self.entries.push(crate::model::Entry {
                        id,
                        task,
                        description,
                        started_at: start,
                        ended_at: Some(end),
                    });
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
                self.view_state = state;
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
