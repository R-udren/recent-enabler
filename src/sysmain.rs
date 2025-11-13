use anyhow::{Context, Result};
use std::path::PathBuf;
use windows::core::PCWSTR;
use windows::Win32::System::Services::*;

const SYSMAIN_SERVICE_NAME: &str = "SysMain";
const ERROR_SERVICE_ALREADY_RUNNING: u32 = 1056;

pub struct PrefetchInfo {
    pub pf_count: usize,
    pub oldest_date: Option<String>,
    pub newest_date: Option<String>,
    pub days_since_last: Option<String>,
}

pub fn get_prefetch_folder() -> Result<PathBuf> {
    let windows_dir = std::env::var("SystemRoot")
        .or_else(|_| std::env::var("windir"))
        .context("Не удалось получить путь к Windows")?;
    Ok(PathBuf::from(windows_dir).join("Prefetch"))
}

pub fn get_prefetch_info() -> Result<PrefetchInfo> {
    let prefetch_path = get_prefetch_folder()?;

    if !prefetch_path.exists() {
        return Ok(PrefetchInfo {
            pf_count: 0,
            oldest_date: None,
            newest_date: None,
            days_since_last: None,
        });
    }

    let entries: Vec<_> = match std::fs::read_dir(&prefetch_path) {
        Ok(entries) => entries.filter_map(|e| e.ok()).collect(),
        Err(_) => {
            return Ok(PrefetchInfo {
                pf_count: 0,
                oldest_date: None,
                newest_date: None,
                days_since_last: None,
            })
        }
    };

    let pf_count = entries
        .iter()
        .filter(|e| {
            e.path()
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.eq_ignore_ascii_case("pf"))
                .unwrap_or(false)
        })
        .count();

    let mut dates: Vec<std::time::SystemTime> = entries
        .iter()
        .filter(|e| {
            e.path()
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.eq_ignore_ascii_case("pf"))
                .unwrap_or(false)
        })
        .filter_map(|e| e.metadata().ok())
        .filter_map(|m| m.modified().ok())
        .collect();

    if dates.is_empty() {
        return Ok(PrefetchInfo {
            pf_count,
            oldest_date: None,
            newest_date: None,
            days_since_last: None,
        });
    }

    dates.sort();

    let format_time = |time: std::time::SystemTime| -> String {
        let datetime: chrono::DateTime<chrono::Local> = time.into();
        datetime.format("%d.%m.%Y %H:%M").to_string()
    };

    let oldest_date = dates.first().map(|t| format_time(*t));
    let newest_date = dates.last().map(|t| format_time(*t));

    let days_since_last = if let Some(&newest_time) = dates.last() {
        let now = std::time::SystemTime::now();
        if let Ok(duration) = now.duration_since(newest_time) {
            let days = duration.as_secs() / 86400;
            if days == 0 {
                Some("сегодня".to_string())
            } else if days == 1 {
                Some("1 день назад".to_string())
            } else {
                Some(format!("{} дн. назад", days))
            }
        } else {
            None
        }
    } else {
        None
    };

    Ok(PrefetchInfo {
        pf_count,
        oldest_date,
        newest_date,
        days_since_last,
    })
}

#[derive(Debug, Clone, PartialEq)]
pub enum ServiceStatus {
    Running,
    Stopped,
    Paused,
    Unknown,
    NotFound,
}

impl ServiceStatus {
    pub fn as_str(&self) -> &str {
        match self {
            ServiceStatus::Running => "Запущена",
            ServiceStatus::Stopped => "Остановлена",
            ServiceStatus::Paused => "Приостановлена",
            ServiceStatus::Unknown => "Неизвестно",
            ServiceStatus::NotFound => "Служба не найдена",
        }
    }
}
pub fn get_sysmain_status() -> Result<ServiceStatus> {
    unsafe {
        let scm = OpenSCManagerW(PCWSTR::null(), PCWSTR::null(), SC_MANAGER_CONNECT)
            .context("Не удалось открыть Service Control Manager")?;

        if scm.is_invalid() {
            return Ok(ServiceStatus::NotFound);
        }

        let service_name: Vec<u16> = SYSMAIN_SERVICE_NAME.encode_utf16().chain(Some(0)).collect();
        let service = OpenServiceW(scm, PCWSTR(service_name.as_ptr()), SERVICE_QUERY_STATUS);

        if service.is_err() {
            let _ = CloseServiceHandle(scm);
            return Ok(ServiceStatus::NotFound);
        }

        let service = service.unwrap();
        let mut status = SERVICE_STATUS::default();
        let result = QueryServiceStatus(service, &mut status);

        let _ = CloseServiceHandle(service);
        let _ = CloseServiceHandle(scm);

        if result.is_err() {
            return Ok(ServiceStatus::Unknown);
        }

        let service_status = match status.dwCurrentState {
            SERVICE_RUNNING => ServiceStatus::Running,
            SERVICE_STOPPED => ServiceStatus::Stopped,
            SERVICE_PAUSED => ServiceStatus::Paused,
            _ => ServiceStatus::Unknown,
        };

        Ok(service_status)
    }
}
#[derive(Debug, Clone, PartialEq)]
pub enum StartupType {
    Automatic,
    Manual,
    Disabled,
    Unknown,
}

impl StartupType {
    pub fn as_str(&self) -> &str {
        match self {
            StartupType::Automatic => "Автоматически",
            StartupType::Manual => "Вручную",
            StartupType::Disabled => "Отключена",
            StartupType::Unknown => "Неизвестно",
        }
    }
}
pub fn get_sysmain_startup_type() -> Result<StartupType> {
    unsafe {
        let scm = OpenSCManagerW(PCWSTR::null(), PCWSTR::null(), SC_MANAGER_CONNECT)
            .context("Не удалось открыть Service Control Manager")?;

        if scm.is_invalid() {
            return Ok(StartupType::Unknown);
        }

        let service_name: Vec<u16> = SYSMAIN_SERVICE_NAME.encode_utf16().chain(Some(0)).collect();

        let service = OpenServiceW(scm, PCWSTR(service_name.as_ptr()), SERVICE_QUERY_CONFIG);

        if service.is_err() {
            let _ = CloseServiceHandle(scm);
            return Ok(StartupType::Unknown);
        }

        let service = service.unwrap();
        let mut bytes_needed = 0u32;
        let _ = QueryServiceConfigW(service, None, 0, &mut bytes_needed);

        let mut buffer: Vec<u8> = vec![0; bytes_needed as usize];
        let config = buffer.as_mut_ptr() as *mut QUERY_SERVICE_CONFIGW;

        let result = QueryServiceConfigW(service, Some(config), bytes_needed, &mut bytes_needed);

        let _ = CloseServiceHandle(service);
        let _ = CloseServiceHandle(scm);

        if result.is_err() {
            return Ok(StartupType::Unknown);
        }

        let startup = match (*config).dwStartType {
            SERVICE_AUTO_START => StartupType::Automatic,
            SERVICE_DEMAND_START => StartupType::Manual,
            SERVICE_DISABLED => StartupType::Disabled,
            _ => StartupType::Unknown,
        };

        Ok(startup)
    }
}
pub fn enable_sysmain() -> Result<()> {
    unsafe {
        let scm = OpenSCManagerW(PCWSTR::null(), PCWSTR::null(), SC_MANAGER_ALL_ACCESS).context(
            "Не удалось открыть Service Control Manager. Требуются права администратора.",
        )?;

        if scm.is_invalid() {
            return Err(anyhow::anyhow!(
                "Не удалось открыть Service Control Manager"
            ));
        }

        let service_name: Vec<u16> = SYSMAIN_SERVICE_NAME.encode_utf16().chain(Some(0)).collect();

        let service = OpenServiceW(
            scm,
            PCWSTR(service_name.as_ptr()),
            SERVICE_CHANGE_CONFIG | SERVICE_START,
        )
        .context("Не удалось открыть службу SysMain. Требуются права администратора.")?;
        let result = ChangeServiceConfigW(
            service,
            ENUM_SERVICE_TYPE(SERVICE_NO_CHANGE),
            SERVICE_AUTO_START,
            SERVICE_ERROR(SERVICE_NO_CHANGE),
            PCWSTR::null(),
            PCWSTR::null(),
            None,
            PCWSTR::null(),
            PCWSTR::null(),
            PCWSTR::null(),
            PCWSTR::null(),
        );

        if result.is_err() {
            let _ = CloseServiceHandle(service);
            let _ = CloseServiceHandle(scm);
            return Err(anyhow::anyhow!("Не удалось изменить конфигурацию службы"));
        }
        let start_result = StartServiceW(service, None);

        let _ = CloseServiceHandle(service);
        let _ = CloseServiceHandle(scm);
        if start_result.is_err() {
            let err = windows::core::Error::from_thread();
            if err.code().0 as u32 != ERROR_SERVICE_ALREADY_RUNNING {
                return Err(anyhow::anyhow!("Не удалось запустить службу: {}", err));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_prefetch_folder() {
        let path = get_prefetch_folder().unwrap();
        assert!(path.to_string_lossy().contains("Prefetch"));
    }
}
