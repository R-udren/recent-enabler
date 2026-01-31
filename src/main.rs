#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod ui;

use iced::Theme;

fn main() -> iced::Result {
    iced::application(app::init, app::update, app::view)
        .theme(|_: &_| Theme::Dark)
        .window(iced::window::Settings {
            size: iced::Size::new(600.0, 800.0),
            ..Default::default()
        })
        .run()
}
