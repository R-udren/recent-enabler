//! Application state and message handling.

use crate::domain::{PrefetchInfo, RecentInfo, SystemRestoreInfo};
use crate::services;
use iced::widget::{column, scrollable, Space};
use iced::{Center, Element, Fill, Task};
use std::process::Command;

#[derive(Debug, Clone)]
pub enum Message {
    LoadedRecent(Result<RecentInfo, String>),
    LoadedPrefetch(Result<PrefetchInfo, String>),
    LoadedSystemRestore(Result<SystemRestoreInfo, String>),
    EnableRecent,
    EnablePrefetch,
    EnableSystemRestore,
    OpenRecentFolder,
    OpenPrefetchFolder,
    RestartAsAdmin,
}

pub struct App {
    pub is_admin: bool,
    pub recent: Option<RecentInfo>,
    pub prefetch: Option<PrefetchInfo>,
    pub system_restore: Option<SystemRestoreInfo>,
}

impl App {
    pub fn new(is_admin: bool) -> (Self, Task<Message>) {
        let app = Self {
            is_admin,
            recent: None,
            prefetch: None,
            system_restore: None,
        };

        let tasks = Task::batch([
            Task::perform(
                async { services::recent::get_info().map_err(|e: crate::domain::AppError| e.to_string()) },
                Message::LoadedRecent
            ),
            Task::perform(
                async { services::prefetch::get_info().map_err(|e: crate::domain::AppError| e.to_string()) },
                Message::LoadedPrefetch
            ),
            Task::perform(
                async { services::system_restore::get_info().map_err(|e: crate::domain::AppError| e.to_string()) },
                Message::LoadedSystemRestore,
            ),
        ]);

        (app, tasks)
    }

    pub fn title(&self) -> String {
        "Recent Files Enabler".to_string()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::LoadedRecent(result) => {
                if let Ok(info) = result {
                    self.recent = Some(info);
                }
                Task::none()
            }
            Message::LoadedPrefetch(result) => {
                if let Ok(info) = result {
                    self.prefetch = Some(info);
                }
                Task::none()
            }
            Message::LoadedSystemRestore(result) => {
                if let Ok(info) = result {
                    self.system_restore = Some(info);
                }
                Task::none()
            }
            Message::EnableRecent => Task::perform(
                async {
                    services::recent::enable().ok();
                    services::recent::get_info().map_err(|e: crate::domain::AppError| e.to_string())
                },
                Message::LoadedRecent,
            ),
            Message::EnablePrefetch => Task::perform(
                async {
                    services::prefetch::enable().ok();
                    services::prefetch::get_info().map_err(|e: crate::domain::AppError| e.to_string())
                },
                Message::LoadedPrefetch,
            ),
            Message::EnableSystemRestore => Task::perform(
                async {
                    services::system_restore::enable("C:\\").ok();
                    services::system_restore::get_info().map_err(|e: crate::domain::AppError| e.to_string())
                },
                Message::LoadedSystemRestore,
            ),
            Message::OpenRecentFolder => {
                let _ = Command::new("explorer")
                    .arg("%APPDATA%\\Microsoft\\Windows\\Recent")
                    .spawn();
                Task::none()
            }
            Message::OpenPrefetchFolder => {
                let _ = Command::new("explorer")
                    .arg("C:\\Windows\\Prefetch")
                    .spawn();
                Task::none()
            }
            Message::RestartAsAdmin => {
                let _ = Command::new("powershell")
                    .arg("-Command")
                    .arg("Start-Process -Verb RunAs -FilePath (Get-Process -Id $PID).Path")
                    .spawn();
                std::process::exit(0);
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        scrollable(
            column![
                Space::new().height(20),
                crate::ui::recent_view::view(
                    self.recent.as_ref(),
                    Message::EnableRecent,
                    Message::OpenRecentFolder
                ),
                Space::new().height(15),
                crate::ui::prefetch_view::view(
                    self.prefetch.as_ref(),
                    self.is_admin,
                    Message::EnablePrefetch,
                    Message::OpenPrefetchFolder,
                    Message::RestartAsAdmin
                ),
                Space::new().height(15),
                crate::ui::system_restore_view::view(
                    self.system_restore.as_ref(),
                    self.is_admin,
                    Message::EnableSystemRestore,
                    Message::RestartAsAdmin
                ),
                Space::new().height(20),
            ]
            .spacing(0)
            .width(Fill)
            .align_x(Center),
        )
        .into()
    }
}
