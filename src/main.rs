#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod domain;
mod repositories;
mod services;
mod ui;

use iced::{window, Size, Theme};

fn main() -> iced::Result {
    let is_admin = unsafe { windows::Win32::UI::Shell::IsUserAnAdmin().as_bool() };

    iced::application(
        move || app::App::new(is_admin),
        app::App::update,
        app::App::view,
    )
    .title(|_: &app::App| "Recent Files Enabler".to_string())
    .theme(|_: &app::App| Theme::Dark)
    .window(window::Settings {
        size: Size::new(600.0, 700.0),
        ..Default::default()
    })
    .run()
}
