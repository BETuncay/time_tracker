use iced::widget::{button, column, container, row, text, text_input, horizontal_space};
use iced::{Element, Length};
use crate::model::{format_duration, format_hm, format_time, Model, ViewState};
use crate::update::Message;

impl Model {
    pub fn view(&self) -> Element<'_, Message> {
        match &self.view_state {
            ViewState::Main => self.view_main(),
            ViewState::ManualEntry => self.view_manual_entry(),
            ViewState::EditEntry(_) => self.view_edit_entry(),
            ViewState::TaskManagement => self.view_task_management(),
            ViewState::Report => text("Report — TODO").into(),
        }
    }

    fn view_main(&self) -> Element<'_, Message> {
        let active_task = self.active.as_ref().map(|t| t.task.as_str());

        // Active timer status
        let status: Element<'_, Message> = if let Some(timer) = &self.active {
            let elapsed = timer.elapsed_start.elapsed().as_secs() as i64;
            column![
                text(format!("Running: {}", timer.task)).size(18),
                text(format_duration(elapsed)).size(32),
                button("Stop").on_press(Message::StopCurrent).style(button::danger),
            ]
            .spacing(8)
            .into()
        } else {
            text("No active timer").size(18).into()
        };

        // Task buttons grid — two per row; active task is highlighted
        let task_buttons: Element<'_, Message> = {
            let mut rows: Vec<Element<'_, Message>> = Vec::new();
            for chunk in self.tasks.chunks(2) {
                let mut r: Vec<Element<'_,Message>> = chunk
                    .iter()
                    .map(|t| {
                        let is_active = active_task == Some(t.as_str());
                        let btn = button(
                            container(text(t.as_str()))
                                .width(Length::Fill)
                                .center_x(Length::Fill),
                        )
                        .on_press(Message::StartTask(t.clone()))
                        .width(Length::Fill)
                        .padding([10, 16]);
                        if is_active {
                            btn.style(button::success).into()
                        } else {
                            btn.style(button::primary).into()
                        }
                    })
                    .collect();
                // Pad last row if odd number of tasks
                if r.len() == 1 {
                    r.push(container(text("")).width(Length::Fill).into());
                }
                rows.push(row(r).spacing(8).into());
            }
            column(rows).spacing(8).into()
        };

        // Entry log
        let log: Element<'_, Message> = {
            let mut items: Vec<Element<'_, Message>> = vec![
                // Header row
                row![
                    text("Task").width(Length::FillPortion(3)),
                    text("Description").width(Length::FillPortion(4)),
                    text("Start").width(Length::FillPortion(2)),
                    text("End").width(Length::FillPortion(2)),
                    text("Dur").width(Length::FillPortion(2)),
                    text("").width(32),
                ]
                .spacing(8)
                .into(),
            ];
            if self.entries.is_empty() {
                items.push(text("No entries yet.").size(14).into());
            } else {
                for entry in &self.entries {
                    let end_str = entry
                        .ended_at
                        .map(format_time)
                        .unwrap_or_else(|| "—".to_string());
                    items.push(
                        row![
                            text(entry.task.as_str()).width(Length::FillPortion(3)),
                            text(entry.description.as_str()).width(Length::FillPortion(4)),
                            text(format_time(entry.started_at)).width(Length::FillPortion(2)),
                            text(end_str).width(Length::FillPortion(2)),
                            text(format_hm(entry.duration_secs())).width(Length::FillPortion(2)),
                            button("Edit")
                                .on_press(Message::ShowView(ViewState::EditEntry(entry.id)))
                                .style(button::secondary)
                                .width(48),
                            button("X")
                                .on_press(Message::DeleteEntry(entry.id))
                                .style(button::danger)
                                .width(32),
                        ]
                        .spacing(8)
                        .into(),
                    );
                }
            }
            column(items).spacing(4).into()
        };

        let bottom_row = row![
            button("+ Add Entry")
                .on_press(Message::ShowView(ViewState::ManualEntry))
                .style(button::secondary),
            horizontal_space(),
            button("Manage Tasks")
                .on_press(Message::ShowView(ViewState::TaskManagement))
                .style(button::secondary),
        ]
        .spacing(8);

        container(
            column![status, task_buttons, log, bottom_row]
                .spacing(24)
                .padding(16),
        )
        .into()
    }

    fn view_manual_entry(&self) -> Element<'_, Message> {
        // Task picker — buttons, one per task, active task highlighted
        let task_picker: Element<'_, Message> = {
            let mut rows: Vec<Element<'_, Message>> = Vec::new();
            for chunk in self.tasks.chunks(2) {
                let r: Vec<Element<'_, Message>> = chunk
                    .iter()
                    .map(|t| {
                        let is_selected = t == &self.form_task;
                        let btn = button(
                            container(text(t.as_str()))
                                .width(Length::Fill)
                                .center_x(Length::Fill),
                        )
                        .on_press(Message::ManualFormTask(t.clone()))
                        .width(Length::Fill)
                        .padding([8, 12]);
                        if is_selected {
                            btn.style(button::success).into()
                        } else {
                            btn.style(button::secondary).into()
                        }
                    })
                    .collect();
                rows.push(row(r).spacing(8).into());
            }
            column(rows).spacing(8).into()
        };

        let error_row: Element<'_, Message> = if let Some(err) = &self.form_error {
            text(err.as_str()).size(13).color([0.9, 0.3, 0.3]).into()
        } else {
            text("").size(13).into()
        };

        let form = column![
            text("New Entry").size(20),
            text("Task").size(13),
            task_picker,
            text("Description").size(13),
            text_input("Optional description", &self.form_desc)
                .on_input(Message::ManualFormDesc)
                .padding([8, 10]),
            row![
                column![
                    text("Start (HH:MM)").size(13),
                    text_input("e.g. 09:00", &self.form_start)
                        .on_input(Message::ManualFormStart)
                        .padding([8, 10]),
                ]
                .spacing(4)
                .width(Length::Fill),
                column![
                    text("End (HH:MM)").size(13),
                    text_input("e.g. 10:30", &self.form_end)
                        .on_input(Message::ManualFormEnd)
                        .padding([8, 10]),
                ]
                .spacing(4)
                .width(Length::Fill),
            ]
            .spacing(12),
            error_row,
            row![
                button("Save").on_press(Message::SubmitManualEntry).style(button::primary),
                button("Cancel")
                    .on_press(Message::ShowView(ViewState::Main))
                    .style(button::secondary),
            ]
            .spacing(8),
        ]
        .spacing(12)
        .padding(16);

        container(form).into()
    }

    fn view_edit_entry(&self) -> Element<'_, Message> {
        let task_picker: Element<'_, Message> = {
            let mut rows: Vec<Element<'_, Message>> = Vec::new();
            for chunk in self.tasks.chunks(2) {
                let r: Vec<Element<'_, Message>> = chunk
                    .iter()
                    .map(|t| {
                        let is_selected = t == &self.form_task;
                        let btn = button(
                            container(text(t.as_str()))
                                .width(Length::Fill)
                                .center_x(Length::Fill),
                        )
                        .on_press(Message::EditFormTask(t.clone()))
                        .width(Length::Fill)
                        .padding([8, 12]);
                        if is_selected {
                            btn.style(button::success).into()
                        } else {
                            btn.style(button::secondary).into()
                        }
                    })
                    .collect();
                rows.push(row(r).spacing(8).into());
            }
            column(rows).spacing(8).into()
        };

        let error_row: Element<'_, Message> = if let Some(err) = &self.form_error {
            text(err.as_str()).size(13).color([0.9, 0.3, 0.3]).into()
        } else {
            text("").size(13).into()
        };

        let form = column![
            text("Edit Entry").size(20),
            text("Task").size(13),
            task_picker,
            text("Description").size(13),
            text_input("Optional description", &self.form_desc)
                .on_input(Message::EditFormDesc)
                .padding([8, 10]),
            row![
                column![
                    text("Start (HH:MM)").size(13),
                    text_input("e.g. 09:00", &self.form_start)
                        .on_input(Message::EditFormStart)
                        .padding([8, 10]),
                ]
                .spacing(4)
                .width(Length::Fill),
                column![
                    text("End (HH:MM)").size(13),
                    text_input("e.g. 10:30", &self.form_end)
                        .on_input(Message::EditFormEnd)
                        .padding([8, 10]),
                ]
                .spacing(4)
                .width(Length::Fill),
            ]
            .spacing(12),
            error_row,
            row![
                button("Save").on_press(Message::SubmitEditEntry).style(button::primary),
                button("Cancel")
                    .on_press(Message::ShowView(ViewState::Main))
                    .style(button::secondary),
            ]
            .spacing(8),
        ]
        .spacing(12)
        .padding(16);

        container(form).into()
    }

    fn view_task_management(&self) -> Element<'_, Message> {
        let mut items: Vec<Element<'_, Message>> = vec![
            text("Manage Tasks").size(20).into(),
        ];

        for (idx, task) in self.tasks.iter().enumerate() {
            let row_el: Element<'_, Message> = if self.task_renaming == Some(idx) {
                row![
                    text_input("Task name", &self.task_rename_text)
                        .on_input(Message::TaskRenameText)
                        .on_submit(Message::TaskConfirmRename(idx))
                        .padding([6, 10])
                        .width(Length::Fill),
                    button("OK")
                        .on_press(Message::TaskConfirmRename(idx))
                        .style(button::primary),
                    button("Cancel")
                        .on_press(Message::TaskCancelRename)
                        .style(button::secondary),
                ]
                .spacing(8)
                .into()
            } else {
                row![
                    text(task.as_str()).width(Length::Fill),
                    button("Rename")
                        .on_press(Message::TaskStartRename(idx))
                        .style(button::secondary),
                    button("Delete")
                        .on_press(Message::TaskDelete(idx))
                        .style(button::danger),
                ]
                .spacing(8)
                .into()
            };
            items.push(row_el);
        }

        // Add-task row
        let add_error: Element<'_, Message> = if let Some(err) = &self.task_new_error {
            text(err.as_str()).size(13).color([0.9, 0.3, 0.3]).into()
        } else {
            text("").size(13).into()
        };

        items.push(
            row![
                text_input("New task name", &self.task_new_name)
                    .on_input(Message::TaskNewName)
                    .on_submit(Message::TaskAdd)
                    .padding([6, 10])
                    .width(Length::Fill),
                button("Add Task")
                    .on_press(Message::TaskAdd)
                    .style(button::primary),
            ]
            .spacing(8)
            .into(),
        );
        items.push(add_error);
        items.push(
            button("Back")
                .on_press(Message::ShowView(ViewState::Main))
                .style(button::secondary)
                .into(),
        );

        container(
            column(items).spacing(10).padding(16),
        )
        .into()
    }
}
