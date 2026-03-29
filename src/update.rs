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
    // edit entry form (reuses form_* fields)
    EditFormTask(String),
    EditFormDesc(String),
    EditFormStart(String),
    EditFormEnd(String),
    SubmitEditEntry,
    // task management
    TaskNewName(String),
    TaskAdd,
    TaskStartRename(usize),
    TaskRenameText(String),
    TaskConfirmRename(usize),
    TaskCancelRename,
    TaskDelete(usize),
}

impl Model {
    pub fn new() -> (Self, Task<Message>) {
        let conn = db::open().expect("failed to open database");
        let tasks = db::load_tasks(&conn).unwrap_or_default();
        let entries = db::load_today(&conn).unwrap_or_default();
        let form_task = tasks.first().cloned().unwrap_or_default();
        let model = Self {
            tasks,
            active: None,
            entries,
            report_entries: Vec::new(),
            view_state: ViewState::Main,
            form_task,
            form_desc: String::new(),
            form_start: String::new(),
            form_end: String::new(),
            form_error: None,
            task_new_name: String::new(),
            task_new_error: None,
            task_renaming: None,
            task_rename_text: String::new(),
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
                match &state {
                    ViewState::ManualEntry => {
                        self.form_task = self.tasks.first().cloned().unwrap_or_default();
                        self.form_desc = String::new();
                        self.form_start = String::new();
                        self.form_end = String::new();
                        self.form_error = None;
                    }
                    ViewState::EditEntry(id) => {
                        if let Some(entry) = self.entries.iter().find(|e| e.id == *id) {
                            use crate::model::format_time;
                            self.form_task = entry.task.clone();
                            self.form_desc = entry.description.clone();
                            self.form_start = format_time(entry.started_at);
                            self.form_end = entry.ended_at.map(format_time).unwrap_or_default();
                            self.form_error = None;
                        }
                    }
                    ViewState::Report => {
                        let conn = db::open().expect("db open");
                        self.report_entries = db::load_week(&conn).unwrap_or_default();
                    }
                    _ => {}
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

            Message::EditFormTask(t) => { self.form_task = t; Task::none() }
            Message::EditFormDesc(d) => { self.form_desc = d; Task::none() }
            Message::EditFormStart(s) => { self.form_start = s; Task::none() }
            Message::EditFormEnd(e) => { self.form_end = e; Task::none() }

            Message::SubmitEditEntry => {
                let id = if let ViewState::EditEntry(id) = self.view_state { id } else { return Task::none(); };
                let start = parse_hhmm_today(&self.form_start);
                let end = parse_hhmm_today(&self.form_end);
                match (start, end) {
                    (Some(s), Some(e)) if e > s => {
                        let conn = db::open().expect("db open");
                        let _ = db::update_entry(&conn, id, &self.form_task, &self.form_desc, s, Some(e));
                        if let Some(entry) = self.entries.iter_mut().find(|e| e.id == id) {
                            entry.task = self.form_task.clone();
                            entry.description = self.form_desc.clone();
                            entry.started_at = s;
                            entry.ended_at = Some(e);
                        }
                        self.entries.sort_by_key(|e| e.started_at);
                        self.view_state = ViewState::Main;
                        self.form_error = None;
                    }
                    (None, _) => self.form_error = Some("Invalid start time (use HH:MM)".into()),
                    (_, None) => self.form_error = Some("Invalid end time (use HH:MM)".into()),
                    _ => self.form_error = Some("End time must be after start time".into()),
                }
                Task::none()
            }

            Message::TaskNewName(s) => {
                self.task_new_name = s;
                self.task_new_error = None;
                Task::none()
            }

            Message::TaskAdd => {
                let name = self.task_new_name.trim().to_string();
                if name.is_empty() {
                    self.task_new_error = Some("Task name cannot be empty".into());
                } else if self.tasks.iter().any(|t| t.eq_ignore_ascii_case(&name)) {
                    self.task_new_error = Some("A task with that name already exists".into());
                } else {
                    let conn = db::open().expect("db open");
                    if let Err(_) = db::insert_task(&conn, &name) {
                        self.task_new_error = Some("Failed to save task".into());
                    } else {
                        self.tasks.push(name);
                        self.task_new_name = String::new();
                        self.task_new_error = None;
                    }
                }
                Task::none()
            }

            Message::TaskStartRename(idx) => {
                if let Some(name) = self.tasks.get(idx) {
                    self.task_rename_text = name.clone();
                    self.task_renaming = Some(idx);
                }
                Task::none()
            }

            Message::TaskRenameText(s) => {
                self.task_rename_text = s;
                Task::none()
            }

            Message::TaskConfirmRename(idx) => {
                let new_name = self.task_rename_text.trim().to_string();
                let old_name = self.tasks.get(idx).cloned().unwrap_or_default();
                if !new_name.is_empty()
                    && !self.tasks.iter().enumerate()
                        .any(|(i, t)| i != idx && t.eq_ignore_ascii_case(&new_name))
                {
                    let conn = db::open().expect("db open");
                    let _ = db::rename_task(&conn, &old_name, &new_name);
                    // Update in-memory entries that referenced old name
                    for entry in self.entries.iter_mut() {
                        if entry.task == old_name {
                            entry.task = new_name.clone();
                        }
                    }
                    // Update active timer task name if needed
                    if let Some(ref mut timer) = self.active {
                        if timer.task == old_name {
                            timer.task = new_name.clone();
                        }
                    }
                    if let Some(task) = self.tasks.get_mut(idx) {
                        *task = new_name;
                    }
                }
                self.task_renaming = None;
                self.task_rename_text = String::new();
                Task::none()
            }

            Message::TaskCancelRename => {
                self.task_renaming = None;
                self.task_rename_text = String::new();
                Task::none()
            }

            Message::TaskDelete(idx) => {
                if idx < self.tasks.len() {
                    let name = self.tasks.remove(idx);
                    let conn = db::open().expect("db open");
                    let _ = db::delete_task(&conn, &name);
                    // Cancel rename if it was pointing at this or a later index
                    if let Some(ri) = self.task_renaming {
                        if ri >= idx {
                            self.task_renaming = None;
                            self.task_rename_text = String::new();
                        }
                    }
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
