mod recent;
mod sysmain;
mod utils;

use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Element, Fill, Task, Theme};

fn main() -> iced::Result {
    iced::application("Recent & SysMain Manager", update, view)
        .theme(|_| Theme::Dark)
        .window(iced::window::Settings {
            size: iced::Size::new(700.0, 650.0),
            ..Default::default()
        })
        .run_with(|| {
            (
                State::new(),
                Task::batch(vec![
                    Task::perform(check_recent_async(), Message::RecentChecked),
                    Task::perform(check_sysmain_async(), Message::SysMainChecked),
                ]),
            )
        })
}

#[derive(Debug, Clone)]
enum Message {
    EnableRecent,
    EnableSysMain,
    Refresh,
    RecentChecked(Result<RecentStatus, String>),
    SysMainChecked(Result<SysMainStatus, String>),
    RecentEnabled(Result<String, String>),
    SysMainEnabled(Result<String, String>),
    OpenRecentFolder,
    OpenPrefetchFolder,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct RecentStatus {
    path: String,
    is_disabled: bool,
    is_empty: bool,
    files_count: usize,
    folder_size: String,
    oldest_file: Option<String>,
    newest_file: Option<String>,
    days_since_last: Option<String>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct SysMainStatus {
    service_status: String,
    startup_type: String,
    is_running: bool,
    is_auto: bool,
    prefetch_path: String,
    prefetch_count: usize,
    oldest_file: Option<String>,
    newest_file: Option<String>,
    days_since_last: Option<String>,
}

#[derive(Default)]
struct State {
    recent_status: Option<RecentStatus>,
    sysmain_status: Option<SysMainStatus>,
    status_message: String,
    is_admin: bool,
}

impl State {
    fn new() -> Self {
        Self {
            is_admin: utils::is_admin(),
            ..Default::default()
        }
    }
}

fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::Refresh => Task::batch(vec![
            Task::perform(check_recent_async(), Message::RecentChecked),
            Task::perform(check_sysmain_async(), Message::SysMainChecked),
        ]),
        Message::EnableRecent => Task::perform(enable_recent_async(), Message::RecentEnabled),
        Message::EnableSysMain => Task::perform(enable_sysmain_async(), Message::SysMainEnabled),
        Message::RecentChecked(result) => {
            match result {
                Ok(status) => {
                    state.recent_status = Some(status);
                    state.status_message = String::new();
                }
                Err(e) => {
                    state.status_message = format!("Ошибка Recent: {}", e);
                }
            }
            Task::none()
        }
        Message::SysMainChecked(result) => {
            match result {
                Ok(status) => {
                    state.sysmain_status = Some(status);
                    state.status_message = String::new();
                }
                Err(e) => {
                    state.status_message = format!("Ошибка Prefetch: {}", e);
                }
            }
            Task::none()
        }
        Message::RecentEnabled(result) => match result {
            Ok(msg) => {
                state.status_message = msg;
                Task::perform(check_recent_async(), Message::RecentChecked)
            }
            Err(e) => {
                state.status_message = format!("Ошибка: {}", e);
                Task::none()
            }
        },
        Message::SysMainEnabled(result) => match result {
            Ok(msg) => {
                state.status_message = msg;
                Task::perform(check_sysmain_async(), Message::SysMainChecked)
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
    }
}

fn view(state: &State) -> Element<'_, Message> {
    let header = row![
        text("Recent & Prefetch Manager")
            .size(26)
            .width(Fill)
            .style(|_theme| text::Style {
                color: Some(iced::Color::from_rgb(0.9, 0.9, 1.0))
            }),
        button("Обновить")
            .on_press(Message::Refresh)
            .padding([8, 16])
            .style(|_theme, status| {
                let base_color = match status {
                    button::Status::Active => iced::Color::from_rgb(0.2, 0.3, 0.4),
                    button::Status::Hovered => iced::Color::from_rgb(0.3, 0.4, 0.5),
                    button::Status::Pressed => iced::Color::from_rgb(0.15, 0.25, 0.35),
                    _ => iced::Color::from_rgb(0.2, 0.3, 0.4),
                };
                button::Style {
                    background: Some(iced::Background::Color(base_color)),
                    text_color: iced::Color::WHITE,
                    border: iced::Border {
                        radius: 6.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            }),
    ]
    .spacing(10)
    .padding(15)
    .align_y(iced::Alignment::Center);

    let admin_badge = container(
        text(if state.is_admin {
            "Администратор"
        } else {
            "Без прав администратора"
        })
        .size(13),
    )
    .padding([6, 12])
    .style(|_theme| container::Style {
        background: Some(iced::Background::Color(if state.is_admin {
            iced::Color::from_rgb(0.15, 0.3, 0.15)
        } else {
            iced::Color::from_rgb(0.3, 0.2, 0.15)
        })),
        border: iced::Border {
            color: if state.is_admin {
                iced::Color::from_rgb(0.3, 0.6, 0.3)
            } else {
                iced::Color::from_rgb(0.6, 0.4, 0.3)
            },
            width: 1.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    });

    let recent_card = if let Some(status) = &state.recent_status {
        let status_text = if status.is_disabled {
            "ОТКЛЮЧЕНА"
        } else {
            "ВКЛЮЧЕНА"
        };

        let mut card_content = column![
            row![
                text("RECENT (Недавние файлы)").size(20),
                Space::with_width(Fill),
                button("Открыть папку")
                    .on_press(Message::OpenRecentFolder)
                    .padding([5, 10]),
            ]
            .align_y(iced::Alignment::Center),
            Space::with_height(15),
            row![
                text("Статус:").width(140).style(|_theme| text::Style {
                    color: Some(iced::Color::from_rgb(0.7, 0.7, 0.7))
                }),
                text(status_text).size(15).style(|_theme| text::Style {
                    color: Some(if status.is_disabled {
                        iced::Color::from_rgb(1.0, 0.4, 0.4)
                    } else {
                        iced::Color::from_rgb(0.4, 1.0, 0.4)
                    })
                }),
            ]
            .spacing(10),
            row![
                text("Файлов:").width(140).style(|_theme| text::Style {
                    color: Some(iced::Color::from_rgb(0.7, 0.7, 0.7))
                }),
                text(format!("{} ({})", status.files_count, status.folder_size)),
            ]
            .spacing(10),
        ]
        .spacing(8)
        .padding(20);

        if let Some(oldest) = &status.oldest_file {
            card_content = card_content.push(
                row![
                    text("Самый старый:")
                        .width(140)
                        .style(|_theme| text::Style {
                            color: Some(iced::Color::from_rgb(0.7, 0.7, 0.7))
                        }),
                    text(oldest).size(13),
                ]
                .spacing(10),
            );
        }

        if let Some(newest) = &status.newest_file {
            let newest_display = if let Some(days) = &status.days_since_last {
                format!("{} ({})", newest, days)
            } else {
                newest.clone()
            };
            card_content = card_content.push(
                row![
                    text("Самый новый:").width(140).style(|_theme| text::Style {
                        color: Some(iced::Color::from_rgb(0.7, 0.7, 0.7))
                    }),
                    text(newest_display).size(13),
                ]
                .spacing(10),
            );
        }

        card_content = card_content.push(
            row![
                text("Путь:").width(140).style(|_theme| text::Style {
                    color: Some(iced::Color::from_rgb(0.7, 0.7, 0.7))
                }),
                text(&status.path).size(11).style(|_theme| text::Style {
                    color: Some(iced::Color::from_rgb(0.6, 0.6, 0.6))
                }),
            ]
            .spacing(10),
        );

        if status.is_disabled {
            card_content = card_content.push(Space::with_height(15)).push(
                container(
                    button("Включить запись Recent")
                        .on_press(Message::EnableRecent)
                        .padding(10),
                )
                .center_x(Fill),
            );
        }

        container(card_content).style(|_theme| container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgb(
                0.15, 0.2, 0.25,
            ))),
            border: iced::Border {
                color: iced::Color::from_rgb(0.3, 0.4, 0.5),
                width: 1.0,
                radius: 8.0.into(),
            },
            ..Default::default()
        })
    } else {
        container(text("Загрузка статуса Recent...").size(16).width(Fill))
            .padding(20)
            .style(container::rounded_box)
    };

    let sysmain_card = if let Some(status) = &state.sysmain_status {
        let service_text = if status.is_running {
            "ЗАПУЩЕНА"
        } else {
            "ОСТАНОВЛЕНА"
        };

        let mut card_content = column![
            row![
                text("PREFETCH (SuperFetch)").size(20),
                Space::with_width(Fill),
                button("Открыть папку")
                    .on_press(Message::OpenPrefetchFolder)
                    .padding([5, 10]),
            ]
            .align_y(iced::Alignment::Center),
            Space::with_height(15),
            row![
                text("Статус службы:")
                    .width(140)
                    .style(|_theme| text::Style {
                        color: Some(iced::Color::from_rgb(0.7, 0.7, 0.7))
                    }),
                text(service_text).size(15).style(|_theme| text::Style {
                    color: Some(if status.is_running {
                        iced::Color::from_rgb(0.4, 1.0, 0.4)
                    } else {
                        iced::Color::from_rgb(1.0, 0.4, 0.4)
                    })
                }),
            ]
            .spacing(10),
            row![
                text("Тип запуска:").width(140).style(|_theme| text::Style {
                    color: Some(iced::Color::from_rgb(0.7, 0.7, 0.7))
                }),
                text(&status.startup_type),
            ]
            .spacing(10),
            row![
                text("Файлов (.pf):")
                    .width(140)
                    .style(|_theme| text::Style {
                        color: Some(iced::Color::from_rgb(0.7, 0.7, 0.7))
                    }),
                text(format!("{}", status.prefetch_count)),
            ]
            .spacing(10),
        ]
        .spacing(8)
        .padding(20);

        if let Some(oldest) = &status.oldest_file {
            card_content = card_content.push(
                row![
                    text("Самый старый:")
                        .width(140)
                        .style(|_theme| text::Style {
                            color: Some(iced::Color::from_rgb(0.7, 0.7, 0.7))
                        }),
                    text(oldest).size(13),
                ]
                .spacing(10),
            );
        }

        if let Some(newest) = &status.newest_file {
            let newest_display = if let Some(days) = &status.days_since_last {
                format!("{} ({})", newest, days)
            } else {
                newest.clone()
            };
            card_content = card_content.push(
                row![
                    text("Самый новый:").width(140).style(|_theme| text::Style {
                        color: Some(iced::Color::from_rgb(0.7, 0.7, 0.7))
                    }),
                    text(newest_display).size(13),
                ]
                .spacing(10),
            );
        }

        card_content = card_content.push(
            row![
                text("Путь:").width(140).style(|_theme| text::Style {
                    color: Some(iced::Color::from_rgb(0.7, 0.7, 0.7))
                }),
                text(&status.prefetch_path)
                    .size(11)
                    .style(|_theme| text::Style {
                        color: Some(iced::Color::from_rgb(0.6, 0.6, 0.6))
                    }),
            ]
            .spacing(10),
        );

        if !status.is_running || !status.is_auto {
            card_content = card_content.push(Space::with_height(15)).push(
                container(
                    button("Включить службу Prefetch")
                        .on_press(Message::EnableSysMain)
                        .padding(10),
                )
                .center_x(Fill),
            );
        }

        container(card_content).style(|_theme| container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgb(
                0.15, 0.25, 0.2,
            ))),
            border: iced::Border {
                color: iced::Color::from_rgb(0.3, 0.5, 0.4),
                width: 1.0,
                radius: 8.0.into(),
            },
            ..Default::default()
        })
    } else {
        container(text("Загрузка статуса Prefetch...").size(16).width(Fill))
            .padding(20)
            .style(container::rounded_box)
    };

    let status_msg = if !state.status_message.is_empty() {
        Some(
            container(text(&state.status_message).size(14))
                .padding(12)
                .style(|_theme| container::Style {
                    background: Some(iced::Background::Color(iced::Color::from_rgb(
                        0.2, 0.25, 0.15,
                    ))),
                    border: iced::Border {
                        color: iced::Color::from_rgb(0.5, 0.6, 0.3),
                        width: 1.0,
                        radius: 6.0.into(),
                    },
                    ..Default::default()
                }),
        )
    } else {
        None
    };

    let hint = if !state.is_admin {
        Some(
            container(
                row![
                    text("⚠").size(16).style(|_theme| text::Style { color: Some(iced::Color::from_rgb(1.0, 0.8, 0.2)) }),
                    Space::with_width(8),
                    text("Для полного доступа к функциям запустите программу с правами администратора").size(12),
                ]
                .align_y(iced::Alignment::Center)
            )
            .padding(12)
            .style(|_theme| container::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgb(0.25, 0.2, 0.15))),
                border: iced::Border {
                    color: iced::Color::from_rgb(0.6, 0.5, 0.3),
                    width: 1.0,
                    radius: 6.0.into(),
                },
                ..Default::default()
            }),
        )
    } else {
        None
    };

    let mut content = column![
        header,
        admin_badge,
        Space::with_height(15),
        recent_card,
        Space::with_height(15),
        sysmain_card,
    ]
    .spacing(5)
    .padding(15);

    if let Some(msg) = status_msg {
        content = content.push(Space::with_height(10)).push(msg);
    }

    if let Some(h) = hint {
        content = content.push(Space::with_height(10)).push(h);
    }

    container(scrollable(content))
        .width(Fill)
        .height(Fill)
        .into()
}

async fn check_recent_async() -> Result<RecentStatus, String> {
    let path = recent::get_recent_folder().map_err(|e| e.to_string())?;
    let is_disabled = recent::is_recent_disabled().map_err(|e| e.to_string())?;
    let is_empty = recent::is_recent_folder_empty().map_err(|e| e.to_string())?;
    let files_count = recent::get_recent_files_count().map_err(|e| e.to_string())?;
    let folder_size_bytes = recent::get_recent_folder_size().map_err(|e| e.to_string())?;
    let (oldest_file, newest_file) = recent::get_recent_file_dates().unwrap_or((None, None));
    let days_since_last = recent::get_days_since_last_recent().unwrap_or(None);

    Ok(RecentStatus {
        path: path.display().to_string(),
        is_disabled,
        is_empty,
        files_count,
        folder_size: utils::format_size(folder_size_bytes),
        oldest_file,
        newest_file,
        days_since_last,
    })
}

async fn check_sysmain_async() -> Result<SysMainStatus, String> {
    let service_status = sysmain::get_sysmain_status().map_err(|e| e.to_string())?;
    let startup_type = sysmain::get_sysmain_startup_type().map_err(|e| e.to_string())?;
    let prefetch_path = sysmain::get_prefetch_folder().map_err(|e| e.to_string())?;
    let prefetch_count = sysmain::get_prefetch_files_count().unwrap_or(0);
    let (oldest_file, newest_file) = sysmain::get_prefetch_file_dates().unwrap_or((None, None));
    let days_since_last = sysmain::get_days_since_last_prefetch().unwrap_or(None);

    let is_running = service_status == sysmain::ServiceStatus::Running;
    let is_auto = startup_type == sysmain::StartupType::Automatic;

    Ok(SysMainStatus {
        service_status: service_status.as_str().to_string(),
        startup_type: startup_type.as_str().to_string(),
        is_running,
        is_auto,
        prefetch_path: prefetch_path.display().to_string(),
        prefetch_count,
        oldest_file,
        newest_file,
        days_since_last,
    })
}

async fn enable_recent_async() -> Result<String, String> {
    let is_disabled = recent::is_recent_disabled().map_err(|e| e.to_string())?;

    if !is_disabled {
        return Ok("Запись в Recent уже включена!".to_string());
    }

    recent::enable_recent().map_err(|e| e.to_string())?;
    Ok("Запись в Recent успешно включена!".to_string())
}

async fn enable_sysmain_async() -> Result<String, String> {
    if !utils::is_admin() {
        return Err("Требуются права администратора для включения службы Prefetch!".to_string());
    }

    let service_status = sysmain::get_sysmain_status().map_err(|e| e.to_string())?;
    let startup_type = sysmain::get_sysmain_startup_type().map_err(|e| e.to_string())?;

    if service_status == sysmain::ServiceStatus::Running
        && startup_type == sysmain::StartupType::Automatic
    {
        return Ok("Служба Prefetch уже включена и запущена!".to_string());
    }

    sysmain::enable_sysmain().map_err(|e| e.to_string())?;
    Ok("Служба Prefetch успешно включена и запущена!".to_string())
}
