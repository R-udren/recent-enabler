use crate::{service, ui, utils};
use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Element, Fill, Task};
use std::time::SystemTime;

#[derive(Debug, Clone)]
pub enum Message {
    EnableRecent,
    EnableSysMain,
    EnableSystemRestore,
    Refresh,
    RecentChecked(Result<RecentStatus, String>),
    SysMainChecked(Result<SysMainStatus, String>),
    SystemRestoreChecked(Result<SystemRestoreStatus, String>),
    RecentEnabled(Result<String, String>),
    SysMainEnabled(Result<String, String>),
    SystemRestoreEnabled(Result<String, String>),
    OpenRecentFolder,
    OpenPrefetchFolder,
    RestartAsAdmin,
}

#[derive(Debug, Clone)]
pub struct RecentStatus {
    pub path: String,
    pub is_disabled: bool,
    pub files_count: usize,
    pub oldest_time: Option<SystemTime>,
    pub newest_time: Option<SystemTime>,
}

#[derive(Debug, Clone)]
pub struct SysMainStatus {
    pub is_running: bool,
    pub is_auto: bool,
    pub startup_type: String,
    pub prefetch_path: String,
    pub prefetch_count: usize,
    pub oldest_time: Option<SystemTime>,
    pub newest_time: Option<SystemTime>,
    pub prefetch_error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SystemRestoreStatus {
    pub is_enabled: bool,
}

#[derive(Default)]
pub struct State {
    pub recent_status: Option<RecentStatus>,
    pub sysmain_status: Option<SysMainStatus>,
    pub system_restore_status: Option<SystemRestoreStatus>,
    pub status_message: String,
    pub is_admin: bool,
}

impl State {
    pub fn new() -> Self {
        Self {
            is_admin: utils::is_admin(),
            ..Default::default()
        }
    }
}

pub fn init() -> (State, Task<Message>) {
    (
        State::new(),
        Task::batch(vec![
            Task::perform(service::check_recent(), Message::RecentChecked),
            Task::perform(service::check_sysmain(), Message::SysMainChecked),
            Task::perform(
                service::check_system_restore(),
                Message::SystemRestoreChecked,
            ),
        ]),
    )
}

pub fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::Refresh => Task::batch(vec![
            Task::perform(service::check_recent(), Message::RecentChecked),
            Task::perform(service::check_sysmain(), Message::SysMainChecked),
            Task::perform(
                service::check_system_restore(),
                Message::SystemRestoreChecked,
            ),
        ]),
        Message::EnableRecent => Task::perform(service::enable_recent(), Message::RecentEnabled),
        Message::EnableSysMain => Task::perform(service::enable_sysmain(), Message::SysMainEnabled),
        Message::EnableSystemRestore => Task::perform(
            service::enable_system_restore(),
            Message::SystemRestoreEnabled,
        ),
        Message::RecentChecked(result) => {
            match result {
                Ok(status) => {
                    state.recent_status = Some(status);
                    state.status_message.clear();
                }
                Err(e) => state.status_message = format!("Ошибка Recent: {}", e),
            }
            Task::none()
        }
        Message::SysMainChecked(result) => {
            match result {
                Ok(status) => {
                    state.sysmain_status = Some(status);
                    state.status_message.clear();
                }
                Err(e) => state.status_message = format!("Ошибка Prefetch: {}", e),
            }
            Task::none()
        }
        Message::SystemRestoreChecked(result) => {
            match result {
                Ok(status) => {
                    state.system_restore_status = Some(status);
                    state.status_message.clear();
                }
                Err(e) => state.status_message = format!("Ошибка System Restore: {}", e),
            }
            Task::none()
        }
        Message::RecentEnabled(result) => match result {
            Ok(msg) => {
                state.status_message = msg;
                Task::perform(service::check_recent(), Message::RecentChecked)
            }
            Err(e) => {
                state.status_message = format!("Ошибка: {}", e);
                Task::none()
            }
        },
        Message::SysMainEnabled(result) => match result {
            Ok(msg) => {
                state.status_message = msg;
                Task::batch(vec![
                    Task::perform(service::check_recent(), Message::RecentChecked),
                    Task::perform(service::check_sysmain(), Message::SysMainChecked),
                ])
            }
            Err(e) => {
                state.status_message = format!("Ошибка: {}", e);
                Task::none()
            }
        },
        Message::SystemRestoreEnabled(result) => match result {
            Ok(msg) => {
                state.status_message = msg;
                Task::perform(
                    service::check_system_restore(),
                    Message::SystemRestoreChecked,
                )
            }
            Err(e) => {
                state.status_message = format!("Ошибка: {}", e);
                Task::none()
            }
        },
        Message::OpenRecentFolder => {
            if let Some(status) = &state.recent_status {
                let _ = std::process::Command::new("explorer")
                    .arg(&status.path)
                    .spawn();
            }
            Task::none()
        }
        Message::OpenPrefetchFolder => {
            if let Some(status) = &state.sysmain_status {
                let _ = std::process::Command::new("explorer")
                    .arg(&status.prefetch_path)
                    .spawn();
            }
            Task::none()
        }
        Message::RestartAsAdmin => {
            if let Ok(exe_path) = std::env::current_exe() {
                let _ = std::process::Command::new("powershell")
                    .args([
                        "-Command",
                        &format!(
                            "Start-Process -FilePath '{}' -Verb RunAs",
                            exe_path.display()
                        ),
                    ])
                    .spawn();
                std::process::exit(0);
            }
            Task::none()
        }
    }
}

pub fn view(state: &State) -> Element<'_, Message> {
    let mut content = column![view_header()].spacing(5).padding(15);

    if !state.is_admin {
        content = content.push(view_admin_hint());
    }

    if !state.status_message.is_empty() {
        content = content.push(view_status_message(&state.status_message));
    }

    content = content
        .push(Space::with_height(15))
        .push(view_recent_card(state.recent_status.as_ref()))
        .push(Space::with_height(15))
        .push(view_sysmain_card(
            state.sysmain_status.as_ref(),
            state.is_admin,
        ))
        .push(Space::with_height(15))
        .push(view_system_restore_card(
            state.system_restore_status.as_ref(),
            state.is_admin,
        ));

    container(scrollable(content))
        .width(Fill)
        .height(Fill)
        .into()
}

fn view_header() -> Element<'static, Message> {
    row![
        text("Recent & Prefetch Manager")
            .size(26)
            .color(iced::Color::from_rgb(0.9, 0.9, 1.0)),
        Space::with_width(Fill),
        button("Обновить")
            .on_press(Message::Refresh)
            .padding([8, 16]),
    ]
    .spacing(10)
    .padding(15)
    .align_y(iced::Alignment::Center)
    .into()
}

fn view_admin_hint() -> Element<'static, Message> {
    container(
        row![
            text("Для полного доступа к функциям запустите программу с правами администратора")
                .size(13)
                .width(Fill),
            ui::restart_button(Message::RestartAsAdmin).padding([6, 12]),
        ]
        .spacing(10)
        .align_y(iced::Alignment::Center),
    )
    .padding(12)
    .style(|_| container::Style {
        background: Some(iced::Background::Color(iced::Color::from_rgb(
            0.25, 0.2, 0.15,
        ))),
        border: iced::Border {
            color: iced::Color::from_rgb(0.6, 0.5, 0.3),
            width: 1.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    })
    .into()
}

fn view_status_message(msg: &str) -> Element<'_, Message> {
    container(text(msg).size(14))
        .padding(12)
        .style(|_| container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgb(
                0.2, 0.25, 0.15,
            ))),
            border: iced::Border {
                color: iced::Color::from_rgb(0.5, 0.6, 0.3),
                width: 1.0,
                radius: 6.0.into(),
            },
            ..Default::default()
        })
        .into()
}

fn view_recent_card(status: Option<&RecentStatus>) -> Element<'_, Message> {
    let Some(status) = status else {
        return container(text("Загрузка статуса Recent...").size(16).width(Fill))
            .padding(20)
            .style(container::rounded_box)
            .into();
    };

    let mut content = column![
        ui::card_header("Recent", Message::OpenRecentFolder),
        ui::info_row(
            "Статус:",
            ui::status_text(
                if status.is_disabled {
                    "ОТКЛЮЧЕНА"
                } else {
                    "ВКЛЮЧЕНА"
                },
                !status.is_disabled
            )
        ),
        ui::info_row("Файлов:", ui::value_text(status.files_count)),
        ui::file_info_rows(&status.oldest_time, &status.newest_time),
        ui::info_row(
            "Путь:",
            text(&status.path)
                .size(12)
                .color(iced::Color::from_rgb(0.6, 0.6, 0.6))
        ),
    ]
    .spacing(10)
    .padding(22);

    if status.is_disabled {
        content = content.push(Space::with_height(15)).push(
            container(
                button("Включить запись Recent")
                    .on_press(Message::EnableRecent)
                    .padding(10),
            )
            .center_x(Fill),
        );
    }

    container(content)
        .style(|theme| {
            ui::card_style(
                theme,
                iced::Color::from_rgb(0.15, 0.2, 0.25),
                iced::Color::from_rgb(0.3, 0.4, 0.5),
            )
        })
        .into()
}

fn view_sysmain_card(status: Option<&SysMainStatus>, is_admin: bool) -> Element<'_, Message> {
    let Some(status) = status else {
        return container(text("Загрузка статуса Prefetch...").size(16).width(Fill))
            .padding(20)
            .style(container::rounded_box)
            .into();
    };

    let mut content = column![
        ui::card_header("Prefetch", Message::OpenPrefetchFolder),
        ui::info_row(
            "Статус службы:",
            ui::status_text(
                if status.is_running {
                    "ЗАПУЩЕНА"
                } else {
                    "ОСТАНОВЛЕНА"
                },
                status.is_running
            )
        ),
        ui::info_row("Тип запуска:", ui::value_text(&status.startup_type)),
    ]
    .spacing(10)
    .padding(22);

    // Show error message if prefetch folder is inaccessible
    if let Some(ref error) = status.prefetch_error {
        content = content.push(
            container(
                text(error)
                    .size(13)
                    .color(iced::Color::from_rgb(1.0, 0.7, 0.3)),
            )
            .padding(10)
            .style(|_| container::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgb(
                    0.25, 0.2, 0.15,
                ))),
                border: iced::Border {
                    color: iced::Color::from_rgb(0.6, 0.5, 0.3),
                    width: 1.0,
                    radius: 6.0.into(),
                },
                ..Default::default()
            }),
        );
    } else {
        content = content
            .push(ui::info_row(
                "Файлов (.pf):",
                ui::value_text(status.prefetch_count),
            ))
            .push(ui::file_info_rows(&status.oldest_time, &status.newest_time));
    }

    content = content.push(ui::info_row(
        "Путь:",
        text(&status.prefetch_path)
            .size(12)
            .color(iced::Color::from_rgb(0.6, 0.6, 0.6)),
    ));

    if !status.is_running || !status.is_auto {
        content = content.push(Space::with_height(15)).push(if is_admin {
            container(
                button("Включить службу Prefetch")
                    .on_press(Message::EnableSysMain)
                    .padding(10),
            )
            .center_x(Fill)
        } else {
            ui::warning_box("Требуются права администратора", Message::RestartAsAdmin)
        });
    }

    container(content)
        .style(|theme| {
            ui::card_style(
                theme,
                iced::Color::from_rgb(0.15, 0.25, 0.2),
                iced::Color::from_rgb(0.3, 0.5, 0.4),
            )
        })
        .into()
}

fn view_system_restore_card(
    status: Option<&SystemRestoreStatus>,
    is_admin: bool,
) -> Element<'_, Message> {
    let Some(status) = status else {
        return container(
            text("Загрузка статуса System Restore...")
                .size(16)
                .width(Fill),
        )
        .padding(20)
        .style(container::rounded_box)
        .width(Fill)
        .into();
    };

    let mut content = column![
        text("System Restore").size(22),
        ui::info_row(
            "Статус на диске C:",
            ui::status_text(
                if status.is_enabled {
                    "ВКЛЮЧЕНА"
                } else {
                    "ОТКЛЮЧЕНА"
                },
                status.is_enabled
            )
        ),
    ]
    .spacing(10)
    .padding(22);

    // Show enable button or admin warning if not enabled
    if !status.is_enabled {
        content = content.push(Space::with_height(15));

        if is_admin {
            content = content.push(
                container(
                    button("Включить System Restore на C:")
                        .on_press(Message::EnableSystemRestore)
                        .padding(10),
                )
                .center_x(Fill),
            );
        } else {
            content = content.push(ui::warning_box(
                "Требуются права администратора",
                Message::RestartAsAdmin,
            ));
        }
    }

    container(content)
        .width(Fill)
        .style(|theme| {
            ui::card_style(
                theme,
                iced::Color::from_rgb(0.2, 0.15, 0.25),
                iced::Color::from_rgb(0.5, 0.3, 0.5),
            )
        })
        .into()
}
