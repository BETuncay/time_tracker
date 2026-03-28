mod db;
mod model;
mod update;
mod view;

use iced::{application, Theme};
use model::Model;

pub fn main() -> iced::Result {
    application("Time Tracker", Model::update, Model::view)
        .theme(|_| Theme::Dark)
        .subscription(Model::subscription)
        .run_with(Model::new)
}
