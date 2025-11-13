#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod recent;
mod sysmain;
mod ui;
mod utils;

use iced::Theme;

fn main() -> iced::Result {
    iced::application("Recent & Prefetch", app::update, app::view)
        .theme(|_| Theme::Dark)
        .window(iced::window::Settings {
            size: iced::Size::new(600.0, 800.0),
            ..Default::default()
        })
        .run_with(app::init)
}
