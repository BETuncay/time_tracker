use iced::widget::{button, column, container, row, text};
use iced::{Element, Length};
use crate::model::{Model, ViewState};
use crate::update::Message;

impl Model {
    pub fn view(&self) -> Element<Message> {
        match &self.view_state {
            ViewState::Main => self.view_main(),
            ViewState::ManualEntry => text("Manual Entry — TODO").into(),
            ViewState::EditEntry(_) => text("Edit Entry — TODO").into(),
            ViewState::TaskManagement => text("Task Management — TODO").into(),
            ViewState::Report => text("Report — TODO").into(),
        }
    }

    fn view_main(&self) -> Element<Message> {
        // Active timer status
        let status: Element<Message> = if let Some(timer) = &self.active {
            let elapsed = timer.elapsed_start.elapsed().as_secs();
            let h = elapsed / 3600;
            let m = (elapsed % 3600) / 60;
            let s = elapsed % 60;
            column![
                text(format!("Running: {}", timer.task)).size(18),
                text(format!("{:02}:{:02}:{:02}", h, m, s)).size(32),
                button("Stop").on_press(Message::StopCurrent),
            ]
            .spacing(8)
            .into()
        } else {
            text("No active timer").size(18).into()
        };

        // Task buttons grid — two per row
        let task_buttons: Element<Message> = {
            let mut rows: Vec<Element<Message>> = Vec::new();
            let mut chunks = self.tasks.chunks(2);
            while let Some(chunk) = chunks.next() {
                let mut r: Vec<Element<Message>> = chunk
                    .iter()
                    .map(|t| {
                        button(text(t.as_str()))
                            .on_press(Message::StartTask(t.clone()))
                            .width(Length::Fill)
                            .into()
                    })
                    .collect();
                // pad last row if odd
                if r.len() == 1 {
                    r.push(container(text("")).width(Length::Fill).into());
                }
                rows.push(row(r).spacing(8).into());
            }
            column(rows).spacing(8).into()
        };

        // Entry log
        let log: Element<Message> = {
            let mut items: Vec<Element<Message>> = vec![
                text("Today's entries").size(16).into(),
            ];
            for entry in &self.entries {
                let dur = entry.ended_at.map(|e| e - entry.started_at).unwrap_or(0);
                let h = dur / 3600;
                let m = (dur % 3600) / 60;
                items.push(
                    row![
                        text(entry.task.as_str()).width(Length::Fill),
                        text(format!("{:02}h {:02}m", h, m)),
                        button("X").on_press(Message::DeleteEntry(entry.id)),
                    ]
                    .spacing(8)
                    .into(),
                );
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
