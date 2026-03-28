use iced::widget::{button, column, container, row, text};
use iced::{Element, Length};
use crate::model::{format_duration, format_hm, Model, ViewState};
use crate::update::Message;

impl Model {
    pub fn view(&self) -> Element<'_, Message> {
        match &self.view_state {
            ViewState::Main => self.view_main(),
            ViewState::ManualEntry => text("Manual Entry — TODO").into(),
            ViewState::EditEntry(_) => text("Edit Entry — TODO").into(),
            ViewState::TaskManagement => text("Task Management — TODO").into(),
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
                text("Today's entries").size(16).into(),
            ];
            if self.entries.is_empty() {
                items.push(text("No entries yet.").size(14).into());
            } else {
                for entry in &self.entries {
                    items.push(
                        row![
                            text(entry.task.as_str()).width(Length::Fill),
                            text(format_hm(entry.duration_secs())),
                            button("X")
                                .on_press(Message::DeleteEntry(entry.id))
                                .style(button::danger),
                        ]
                        .spacing(8)
                        .into(),
                    );
                }
            }
            column(items).spacing(4).into()
        };

        container(
            column![status, task_buttons, log]
                .spacing(24)
                .padding(16),
        )
        .into()
    }
}
