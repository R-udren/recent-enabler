mod recent;
mod sysmain;
mod utils;

use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Element, Fill, Task, Theme};

fn main() -> iced::Result {
    iced::application("Recent & SysMain Manager", update, view)
        .theme(|_| Theme::Dark)
        .run_with(|| (State::new(), Task::none()))
}

#[derive(Debug, Clone)]
enum Message {
    CheckRecent,
    CheckSysMain,
    EnableRecent,
    EnableSysMain,
    RefreshAll,
    RecentChecked(Result<RecentStatus, String>),
    SysMainChecked(Result<SysMainStatus, String>),
    RecentEnabled(Result<String, String>),
    SysMainEnabled(Result<String, String>),
}

#[derive(Debug, Clone)]
struct RecentStatus {
    path: String,
    is_disabled: bool,
    is_empty: bool,
    files_count: usize,
    folder_size: String,
}

#[derive(Debug, Clone)]
struct SysMainStatus {
    service_status: String,
    startup_type: String,
    is_running: bool,
    is_auto: bool,
    prefetch_path: String,
    prefetch_count: usize,
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
        Message::CheckRecent => Task::perform(check_recent_async(), Message::RecentChecked),
        Message::CheckSysMain => Task::perform(check_sysmain_async(), Message::SysMainChecked),
        Message::EnableRecent => Task::perform(enable_recent_async(), Message::RecentEnabled),
        Message::EnableSysMain => Task::perform(enable_sysmain_async(), Message::SysMainEnabled),
        Message::RefreshAll => Task::batch(vec![
            Task::perform(check_recent_async(), Message::RecentChecked),
            Task::perform(check_sysmain_async(), Message::SysMainChecked),
        ]),
        Message::RecentChecked(result) => {
            match result {
                Ok(status) => {
                    state.recent_status = Some(status);
                    state.status_message = "✅ Статус Recent обновлен".to_string();
                }
                Err(e) => {
                    state.status_message = format!("❌ Ошибка: {}", e);
                }
            }
            Task::none()
        }
        Message::SysMainChecked(result) => {
            match result {
                Ok(status) => {
                    state.sysmain_status = Some(status);
                    state.status_message = "✅ Статус SysMain обновлен".to_string();
                }
                Err(e) => {
                    state.status_message = format!("❌ Ошибка: {}", e);
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
                state.status_message = format!("❌ Ошибка: {}", e);
                Task::none()
            }
        },
        Message::SysMainEnabled(result) => match result {
            Ok(msg) => {
                state.status_message = msg;
                Task::perform(check_sysmain_async(), Message::SysMainChecked)
            }
            Err(e) => {
                state.status_message = format!("❌ Ошибка: {}", e);
                Task::none()
            }
        },
    }
}

fn view(state: &State) -> Element<'_, Message> {
    let title = text("Recent & SysMain Manager").size(32);

    let admin_status = if state.is_admin {
        text("✅ Запущено с правами администратора")
    } else {
        text("⚠️ Запущено без прав администратора")
    };

    let buttons = row![
        button("🔄 Обновить все").on_press(Message::RefreshAll),
        Space::with_width(10),
        button("📁 Проверить Recent").on_press(Message::CheckRecent),
        Space::with_width(10),
        button("⚙️ Проверить SysMain").on_press(Message::CheckSysMain),
    ]
    .padding(10)
    .spacing(10);

    let action_buttons = row![
        button("✅ Включить Recent").on_press(Message::EnableRecent),
        Space::with_width(10),
        button("✅ Включить SysMain").on_press(Message::EnableSysMain),
    ]
    .padding(10)
    .spacing(10);

    let status_text = text(&state.status_message).size(16);

    let mut content = column![
        title,
        admin_status,
        Space::with_height(20),
        buttons,
        action_buttons,
        Space::with_height(10),
        status_text
    ]
    .padding(20)
    .spacing(10);

    if let Some(status) = &state.recent_status {
        let recent_section = column![
            text("📁 Статус Recent").size(24),
            text(format!("Путь: {}", status.path)),
            text(format!(
                "Запись: {}",
                if status.is_disabled {
                    "❌ Отключена"
                } else {
                    "✅ Включена"
                }
            )),
            text(format!(
                "Папка: {}",
                if status.is_empty {
                    "⚠️ Пустая"
                } else {
                    "✅ Содержит файлы"
                }
            )),
            text(format!("Количество файлов: {}", status.files_count)),
            text(format!("Размер папки: {}", status.folder_size)),
        ]
        .spacing(5)
        .padding(10);

        content = content.push(Space::with_height(20)).push(recent_section);
    }

    if let Some(status) = &state.sysmain_status {
        let sysmain_section = column![
            text("⚙️ Статус SysMain").size(24),
            text(format!(
                "Статус службы: {} {}",
                if status.is_running { "✅" } else { "❌" },
                status.service_status
            )),
            text(format!(
                "Тип запуска: {} {}",
                if status.is_auto { "✅" } else { "⚠️" },
                status.startup_type
            )),
            text(format!("Папка Prefetch: {}", status.prefetch_path)),
            text(format!("Файлов .pf: {}", status.prefetch_count)),
        ]
        .spacing(5)
        .padding(10);

        content = content.push(Space::with_height(20)).push(sysmain_section);
    }

    container(scrollable(content))
        .width(Fill)
        .height(Fill)
        .padding(20)
        .into()
}

async fn check_recent_async() -> Result<RecentStatus, String> {
    let path = recent::get_recent_folder().map_err(|e| e.to_string())?;
    let is_disabled = recent::is_recent_disabled().map_err(|e| e.to_string())?;
    let is_empty = recent::is_recent_folder_empty().map_err(|e| e.to_string())?;
    let files_count = recent::get_recent_files_count().map_err(|e| e.to_string())?;
    let folder_size_bytes = recent::get_recent_folder_size().map_err(|e| e.to_string())?;

    Ok(RecentStatus {
        path: path.display().to_string(),
        is_disabled,
        is_empty,
        files_count,
        folder_size: utils::format_size(folder_size_bytes),
    })
}

async fn check_sysmain_async() -> Result<SysMainStatus, String> {
    let service_status = sysmain::get_sysmain_status().map_err(|e| e.to_string())?;
    let startup_type = sysmain::get_sysmain_startup_type().map_err(|e| e.to_string())?;
    let prefetch_path = sysmain::get_prefetch_folder().map_err(|e| e.to_string())?;
    let prefetch_count = sysmain::get_prefetch_files_count().unwrap_or(0);

    let is_running = service_status == sysmain::ServiceStatus::Running;
    let is_auto = startup_type == sysmain::StartupType::Automatic;

    Ok(SysMainStatus {
        service_status: service_status.as_str().to_string(),
        startup_type: startup_type.as_str().to_string(),
        is_running,
        is_auto,
        prefetch_path: prefetch_path.display().to_string(),
        prefetch_count,
    })
}

async fn enable_recent_async() -> Result<String, String> {
    let is_disabled = recent::is_recent_disabled().map_err(|e| e.to_string())?;

    if !is_disabled {
        return Ok("ℹ️ Запись в Recent уже включена!".to_string());
    }

    recent::enable_recent().map_err(|e| e.to_string())?;
    Ok("✅ Запись в Recent успешно включена!".to_string())
}

async fn enable_sysmain_async() -> Result<String, String> {
    if !utils::is_admin() {
        return Err("❌ Требуются права администратора для включения службы SysMain!".to_string());
    }

    let service_status = sysmain::get_sysmain_status().map_err(|e| e.to_string())?;
    let startup_type = sysmain::get_sysmain_startup_type().map_err(|e| e.to_string())?;

    if service_status == sysmain::ServiceStatus::Running
        && startup_type == sysmain::StartupType::Automatic
    {
        return Ok("ℹ️ Служба SysMain уже включена и запущена!".to_string());
    }

    sysmain::enable_sysmain().map_err(|e| e.to_string())?;
    Ok("✅ Служба SysMain успешно включена и запущена!".to_string())
}
