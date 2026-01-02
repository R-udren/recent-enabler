//! Application logic and state management.
//!
//! This module handles the core application state and message handling,
//! completely separated from UI rendering concerns.

use crate::domain::{
    CheckSeverity, OperationResult, RecentCheckResult, RecentInfo, RecentStatus, SysMainInfo,
    SystemRestoreInfo,
};
use crate::{recent, sysmain, system_restore, ui, utils};
use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Color, Element, Fill, Task};

// =============================================================================
// Messages
// =============================================================================

#[derive(Debug, Clone)]
pub enum Message {
    // Actions
    EnableRecent,
    EnableSysMain,
    EnableSystemRestore,
    Refresh,
    OpenRecentFolder,
    OpenPrefetchFolder,
    RestartAsAdmin,
    ToggleRecentDetails,
    TogglePrefetchDetails,

    // Status responses
    RecentChecked(Result<RecentInfo, String>),
    SysMainChecked(Result<SysMainInfo, String>),
    SystemRestoreChecked(Result<SystemRestoreInfo, String>),

    // Operation responses
    RecentEnabled(Result<OperationResult, String>),
    SysMainEnabled(Result<OperationResult, String>),
    SystemRestoreEnabled(Result<OperationResult, String>),
}

// =============================================================================
// Application State
// =============================================================================

#[derive(Default)]
pub struct State {
    // Status data
    pub recent_info: Option<RecentInfo>,
    pub sysmain_info: Option<SysMainInfo>,
    pub system_restore_info: Option<SystemRestoreInfo>,

    // UI state
    pub status_message: String,
    pub is_admin: bool,
    pub show_recent_details: bool,
    pub show_prefetch_details: bool,

    // Operation state
    pub recent_loading: bool,
    pub sysmain_loading: bool,
    pub system_restore_loading: bool,
}

impl State {
    pub fn new() -> Self {
        Self {
            is_admin: utils::is_admin(),
            ..Default::default()
        }
    }
}

// =============================================================================
// Initialization
// =============================================================================

pub fn init() -> (State, Task<Message>) {
    (
        State::new(),
        Task::batch(vec![
            Task::perform(check_recent_async(), Message::RecentChecked),
            Task::perform(check_sysmain_async(), Message::SysMainChecked),
            Task::perform(check_system_restore_async(), Message::SystemRestoreChecked),
        ]),
    )
}

// =============================================================================
// Update Logic
// =============================================================================

pub fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        // === Action Messages ===
        Message::Refresh => {
            state.recent_loading = true;
            state.sysmain_loading = true;
            state.system_restore_loading = true;
            state.status_message.clear();
            Task::batch(vec![
                Task::perform(check_recent_async(), Message::RecentChecked),
                Task::perform(check_sysmain_async(), Message::SysMainChecked),
                Task::perform(check_system_restore_async(), Message::SystemRestoreChecked),
            ])
        }

        Message::EnableRecent => {
            state.recent_loading = true;
            Task::perform(enable_recent_async(), Message::RecentEnabled)
        }

        Message::EnableSysMain => {
            state.sysmain_loading = true;
            Task::perform(enable_sysmain_async(), Message::SysMainEnabled)
        }

        Message::EnableSystemRestore => {
            state.system_restore_loading = true;
            Task::perform(enable_system_restore_async(), Message::SystemRestoreEnabled)
        }

        Message::ToggleRecentDetails => {
            state.show_recent_details = !state.show_recent_details;
            Task::none()
        }

        Message::TogglePrefetchDetails => {
            state.show_prefetch_details = !state.show_prefetch_details;
            Task::none()
        }

        Message::OpenRecentFolder => {
            if let Some(info) = &state.recent_info {
                let _ = std::process::Command::new("explorer")
                    .arg(&info.path)
                    .spawn();
            }
            Task::none()
        }

        Message::OpenPrefetchFolder => {
            if let Some(info) = &state.sysmain_info {
                let _ = std::process::Command::new("explorer")
                    .arg(&info.prefetch_info.path)
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

        // === Status Response Messages ===
        Message::RecentChecked(result) => {
            state.recent_loading = false;
            match result {
                Ok(info) => {
                    state.recent_info = Some(info);
                }
                Err(e) => {
                    state.status_message = format!("Ошибка Recent: {}", e);
                }
            }
            Task::none()
        }

        Message::SysMainChecked(result) => {
            state.sysmain_loading = false;
            match result {
                Ok(info) => {
                    state.sysmain_info = Some(info);
                }
                Err(e) => {
                    state.status_message = format!("Ошибка Prefetch: {}", e);
                }
            }
            Task::none()
        }

        Message::SystemRestoreChecked(result) => {
            state.system_restore_loading = false;
            match result {
                Ok(info) => {
                    state.system_restore_info = Some(info);
                }
                Err(e) => {
                    state.status_message = format!("Ошибка System Restore: {}", e);
                }
            }
            Task::none()
        }

        // === Operation Response Messages ===
        Message::RecentEnabled(result) => {
            state.recent_loading = false;
            match result {
                Ok(op_result) => {
                    state.status_message = op_result.message;
                    Task::perform(check_recent_async(), Message::RecentChecked)
                }
                Err(e) => {
                    state.status_message = format!("Ошибка: {}", e);
                    Task::none()
                }
            }
        }

        Message::SysMainEnabled(result) => {
            state.sysmain_loading = false;
            match result {
                Ok(op_result) => {
                    state.status_message = op_result.message;
                    Task::perform(check_sysmain_async(), Message::SysMainChecked)
                }
                Err(e) => {
                    state.status_message = format!("Ошибка: {}", e);
                    Task::none()
                }
            }
        }

        Message::SystemRestoreEnabled(result) => {
            state.system_restore_loading = false;
            match result {
                Ok(op_result) => {
                    state.status_message = op_result.message;
                    Task::perform(check_system_restore_async(), Message::SystemRestoreChecked)
                }
                Err(e) => {
                    state.status_message = format!("Ошибка: {}", e);
                    Task::none()
                }
            }
        }
    }
}

// =============================================================================
// View Logic
// =============================================================================

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
        .push(view_recent_card(
            state.recent_info.as_ref(),
            state.show_recent_details,
        ))
        .push(Space::with_height(15))
        .push(view_sysmain_card(
            state.sysmain_info.as_ref(),
            state.is_admin,
            state.show_prefetch_details,
        ))
        .push(Space::with_height(15))
        .push(view_system_restore_card(
            state.system_restore_info.as_ref(),
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
            .color(Color::from_rgb(0.9, 0.9, 1.0)),
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
        background: Some(iced::Background::Color(Color::from_rgb(0.25, 0.2, 0.15))),
        border: iced::Border {
            color: Color::from_rgb(0.6, 0.5, 0.3),
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
            background: Some(iced::Background::Color(Color::from_rgb(0.2, 0.25, 0.15))),
            border: iced::Border {
                color: Color::from_rgb(0.5, 0.6, 0.3),
                width: 1.0,
                radius: 6.0.into(),
            },
            ..Default::default()
        })
        .into()
}

// =============================================================================
// Recent Card
// =============================================================================

fn view_recent_card(info: Option<&RecentInfo>, show_details: bool) -> Element<'_, Message> {
    let Some(info) = info else {
        return container(text("Загрузка статуса Recent...").size(16).width(Fill))
            .padding(20)
            .style(container::rounded_box)
            .into();
    };

    let (status_text_val, status_is_good) = get_recent_status_display(&info.status);

    let mut content = column![
        ui::card_header("Recent", Message::OpenRecentFolder),
        ui::info_row(
            "Статус:",
            ui::status_text_owned(status_text_val, status_is_good)
        ),
        ui::info_row("Файлов:", ui::value_text(info.lnk_count)),
        ui::file_info_rows(&info.oldest_time, &info.newest_time),
        ui::info_row(
            "Путь:",
            text(&info.path)
                .size(12)
                .color(Color::from_rgb(0.6, 0.6, 0.6))
        ),
    ]
    .spacing(10)
    .padding(22);

    // Show partial status details if applicable
    if let RecentStatus::PartiallyEnabled {
        disabled_features, ..
    } = &info.status
    {
        content = content.push(
            container(
                column![
                    text("Отключенные функции:")
                        .size(13)
                        .color(Color::from_rgb(1.0, 0.7, 0.3)),
                    text(disabled_features.join(", "))
                        .size(12)
                        .color(Color::from_rgb(0.8, 0.6, 0.4)),
                ]
                .spacing(4),
            )
            .padding(10)
            .style(|_| container::Style {
                background: Some(iced::Background::Color(Color::from_rgb(0.25, 0.2, 0.15))),
                border: iced::Border {
                    color: Color::from_rgb(0.6, 0.5, 0.3),
                    width: 1.0,
                    radius: 6.0.into(),
                },
                ..Default::default()
            }),
        );
    }

    // Show policy block warning
    if let RecentStatus::DisabledByPolicy { policy_sources } = &info.status {
        content = content.push(
            container(
                column![
                    text("⚠ Заблокировано групповой политикой:")
                        .size(13)
                        .color(Color::from_rgb(1.0, 0.5, 0.3)),
                    text(policy_sources.join(", "))
                        .size(12)
                        .color(Color::from_rgb(0.8, 0.5, 0.3)),
                ]
                .spacing(4),
            )
            .padding(10)
            .style(|_| container::Style {
                background: Some(iced::Background::Color(Color::from_rgb(0.3, 0.15, 0.15))),
                border: iced::Border {
                    color: Color::from_rgb(0.7, 0.3, 0.3),
                    width: 1.0,
                    radius: 6.0.into(),
                },
                ..Default::default()
            }),
        );
    }

    // Details toggle button
    content = content.push(
        button(if show_details {
            "Скрыть детали"
        } else {
            "Показать детали"
        })
        .on_press(Message::ToggleRecentDetails)
        .padding([4, 8]),
    );

    // Show detailed checks if expanded
    if show_details {
        content = content.push(view_recent_checks(&info.checks));
    }

    // Enable button if not fully enabled and not policy blocked
    if !info.status.is_enabled() && !info.status.is_policy_blocked() {
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
                Color::from_rgb(0.15, 0.2, 0.25),
                Color::from_rgb(0.3, 0.4, 0.5),
            )
        })
        .into()
}

fn view_recent_checks(checks: &[RecentCheckResult]) -> Element<'_, Message> {
    let mut col = column![text("Детальные проверки:").size(14)].spacing(6);

    for result in checks {
        let check = &result.check;
        let status_icon = if check.is_enabled() { "✓" } else { "✗" };
        let status_color = if check.is_enabled() {
            Color::from_rgb(0.4, 0.8, 0.4)
        } else {
            Color::from_rgb(0.8, 0.4, 0.4)
        };

        let severity_badge = match check.severity {
            CheckSeverity::Critical => ("!", Color::from_rgb(1.0, 0.3, 0.3)),
            CheckSeverity::Important => ("●", Color::from_rgb(1.0, 0.7, 0.3)),
            CheckSeverity::Minor => ("○", Color::from_rgb(0.6, 0.6, 0.6)),
        };

        let policy_badge = if check.is_policy { " [GPO]" } else { "" };

        col = col.push(
            row![
                text(severity_badge.0)
                    .size(12)
                    .color(severity_badge.1)
                    .width(15),
                text(status_icon).size(12).color(status_color).width(15),
                text(format!("{}{}", check.source, policy_badge))
                    .size(12)
                    .color(Color::from_rgb(0.7, 0.7, 0.7)),
            ]
            .spacing(5),
        );
    }

    container(col)
        .padding(10)
        .style(|_| container::Style {
            background: Some(iced::Background::Color(Color::from_rgb(0.1, 0.12, 0.15))),
            border: iced::Border {
                color: Color::from_rgb(0.25, 0.3, 0.35),
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        })
        .into()
}

fn get_recent_status_display(status: &RecentStatus) -> (&'static str, bool) {
    match status {
        RecentStatus::FullyEnabled => ("ВКЛЮЧЕНА", true),
        RecentStatus::FullyDisabled => ("ОТКЛЮЧЕНА", false),
        RecentStatus::PartiallyEnabled { .. } => ("ЧАСТИЧНО", false),
        RecentStatus::DisabledByPolicy { .. } => ("ЗАБЛОКИРОВАНА", false),
    }
}

// =============================================================================
// SysMain Card
// =============================================================================

fn view_sysmain_card(
    info: Option<&SysMainInfo>,
    is_admin: bool,
    show_details: bool,
) -> Element<'_, Message> {
    let Some(info) = info else {
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
                info.service_status.as_str(),
                info.service_status.is_running()
            )
        ),
        ui::info_row("Тип запуска:", ui::value_text(info.startup_type.as_str())),
        ui::info_row(
            "Prefetcher:",
            ui::status_text(
                info.prefetcher_mode.as_str(),
                info.prefetcher_mode.is_enabled()
            )
        ),
    ]
    .spacing(10)
    .padding(22);

    // Show prefetch folder info
    let prefetch = &info.prefetch_info;
    if prefetch.folder_accessible {
        content = content
            .push(ui::info_row(
                "Файлов (.pf):",
                ui::value_text(prefetch.pf_count),
            ))
            .push(ui::file_info_rows(
                &prefetch.oldest_time,
                &prefetch.newest_time,
            ));
    } else if let Some(ref error) = prefetch.error_message {
        content = content.push(
            container(text(error).size(13).color(Color::from_rgb(1.0, 0.7, 0.3)))
                .padding(10)
                .style(|_| container::Style {
                    background: Some(iced::Background::Color(Color::from_rgb(0.25, 0.2, 0.15))),
                    border: iced::Border {
                        color: Color::from_rgb(0.6, 0.5, 0.3),
                        width: 1.0,
                        radius: 6.0.into(),
                    },
                    ..Default::default()
                }),
        );
    }

    content = content.push(ui::info_row(
        "Путь:",
        text(&prefetch.path)
            .size(12)
            .color(Color::from_rgb(0.6, 0.6, 0.6)),
    ));

    // Details toggle
    content = content.push(
        button(if show_details {
            "Скрыть детали"
        } else {
            "Показать детали"
        })
        .on_press(Message::TogglePrefetchDetails)
        .padding([4, 8]),
    );

    if show_details {
        content = content.push(view_prefetch_details(info));
    }

    // Enable button if not fully enabled
    if !info.is_fully_enabled() {
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
                Color::from_rgb(0.15, 0.25, 0.2),
                Color::from_rgb(0.3, 0.5, 0.4),
            )
        })
        .into()
}

fn view_prefetch_details(info: &SysMainInfo) -> Element<'_, Message> {
    let issues = info.get_issues();

    let mut col = column![text("Детали конфигурации:").size(14)].spacing(6);

    col = col.push(
        row![
            text("Superfetch:").size(12).width(120),
            text(info.superfetch_mode.as_str()).size(12).color(
                if info.superfetch_mode.is_enabled() {
                    Color::from_rgb(0.4, 0.8, 0.4)
                } else {
                    Color::from_rgb(0.8, 0.4, 0.4)
                }
            ),
        ]
        .spacing(10),
    );

    if !issues.is_empty() {
        col = col.push(
            text("Проблемы:")
                .size(12)
                .color(Color::from_rgb(1.0, 0.7, 0.3)),
        );
        for issue in issues {
            col = col.push(
                text(format!("• {}", issue))
                    .size(11)
                    .color(Color::from_rgb(0.8, 0.6, 0.4)),
            );
        }
    }

    container(col)
        .padding(10)
        .style(|_| container::Style {
            background: Some(iced::Background::Color(Color::from_rgb(0.1, 0.14, 0.12))),
            border: iced::Border {
                color: Color::from_rgb(0.25, 0.35, 0.3),
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        })
        .into()
}

// =============================================================================
// System Restore Card
// =============================================================================

fn view_system_restore_card(
    info: Option<&SystemRestoreInfo>,
    is_admin: bool,
) -> Element<'_, Message> {
    let Some(info) = info else {
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

    let c_drive_enabled = info.is_c_drive_enabled();

    let mut content = column![
        text("System Restore").size(22),
        ui::info_row(
            "Глобальный статус:",
            ui::status_text(
                if info.global_enabled {
                    "ВКЛЮЧЕНА"
                } else {
                    "ОТКЛЮЧЕНА"
                },
                info.global_enabled
            )
        ),
        ui::info_row(
            "Диск C:",
            ui::status_text(
                if c_drive_enabled {
                    "ЗАЩИЩЁН"
                } else {
                    "НЕ ЗАЩИЩЁН"
                },
                c_drive_enabled
            )
        ),
    ]
    .spacing(10)
    .padding(22);

    // Show available methods
    if let Some(method) = info.preferred_method {
        content = content.push(ui::info_row(
            "Метод:",
            text(method.as_str())
                .size(14)
                .color(Color::from_rgb(0.6, 0.6, 0.6)),
        ));
    }

    // Enable button if not enabled on C:
    if !c_drive_enabled {
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
                Color::from_rgb(0.2, 0.15, 0.25),
                Color::from_rgb(0.5, 0.3, 0.5),
            )
        })
        .into()
}

// =============================================================================
// Async Operations
// =============================================================================

async fn check_recent_async() -> Result<RecentInfo, String> {
    recent::get_recent_info().map_err(|e| e.to_string())
}

async fn check_sysmain_async() -> Result<SysMainInfo, String> {
    sysmain::get_sysmain_info().map_err(|e| e.to_string())
}

async fn check_system_restore_async() -> Result<SystemRestoreInfo, String> {
    system_restore::get_system_restore_info().map_err(|e| e.to_string())
}

async fn enable_recent_async() -> Result<OperationResult, String> {
    recent::enable_recent().map_err(|e| e.to_string())
}

async fn enable_sysmain_async() -> Result<OperationResult, String> {
    if !utils::is_admin() {
        return Ok(OperationResult::failure(
            "Требуются права администратора для включения службы Prefetch!",
        )
        .requires_admin());
    }

    let info = sysmain::get_sysmain_info().map_err(|e| e.to_string())?;

    if info.is_fully_enabled() {
        return Ok(OperationResult::success(
            "Служба Prefetch уже включена и запущена!",
        ));
    }

    sysmain::enable_sysmain().map_err(|e| e.to_string())
}

async fn enable_system_restore_async() -> Result<OperationResult, String> {
    if !utils::is_admin() {
        return Ok(OperationResult::failure(
            "Требуются права администратора для включения System Restore!",
        )
        .requires_admin());
    }

    system_restore::enable_system_restore("C:").map_err(|e| e.to_string())
}
