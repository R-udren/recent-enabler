use anyhow::{Context, Result};
use std::io::ErrorKind;
use std::path::PathBuf;
use windows::core::PCWSTR;
use windows::Win32::System::Services::*;

const SYSMAIN_SERVICE_NAME: &str = "SysMain";
const ERROR_SERVICE_ALREADY_RUNNING: u32 = 1056;

pub struct PrefetchInfo {
    pub pf_count: usize,
    pub oldest_time: Option<std::time::SystemTime>,
    pub newest_time: Option<std::time::SystemTime>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ServiceStatus {
    Running,
    Stopped,
    Paused,
    Unknown,
    NotFound,
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

// === Path and folder operations ===

pub fn get_prefetch_folder() -> Result<PathBuf> {
    let windows_dir = std::env::var("SystemRoot")
        .or_else(|_| std::env::var("windir"))
        .context("Не удалось получить путь к Windows")?;
    Ok(PathBuf::from(windows_dir).join("Prefetch"))
}

fn scan_prefetch_directory(prefetch_path: &PathBuf) -> Result<Vec<std::fs::DirEntry>> {
    let entries = match std::fs::read_dir(prefetch_path) {
        Ok(iter) => iter,
        Err(err) => match err.kind() {
            ErrorKind::PermissionDenied => {
                return Err(anyhow::anyhow!(
                    "Нет доступа к папке Prefetch. Запустите программу с правами администратора."
                ));
            }
            _ => return Err(err).context("Не удалось прочитать папку Prefetch"),
        },
    };

    Ok(entries.filter_map(|e| e.ok()).collect())
}

fn count_pf_files(entries: &[std::fs::DirEntry]) -> usize {
    entries
        .iter()
        .filter(|entry| {
            entry
                .path()
                .extension()
                .map(|ext| ext.eq_ignore_ascii_case("pf"))
                .unwrap_or(false)
        })
        .count()
}

fn collect_pf_dates(entries: &[std::fs::DirEntry]) -> Vec<std::time::SystemTime> {
    entries
        .iter()
        .filter(|entry| {
            entry
                .path()
                .extension()
                .map(|ext| ext.eq_ignore_ascii_case("pf"))
                .unwrap_or(false)
        })
        .filter_map(|entry| entry.metadata().ok())
        .filter_map(|metadata| metadata.modified().ok())
        .collect()
}

fn get_oldest_newest_dates(
    mut dates: Vec<std::time::SystemTime>,
) -> (Option<std::time::SystemTime>, Option<std::time::SystemTime>) {
    if dates.is_empty() {
        return (None, None);
    }
    dates.sort();
    (dates.first().copied(), dates.last().copied())
}

pub fn get_prefetch_info() -> Result<PrefetchInfo> {
    let prefetch_path = get_prefetch_folder()?;

    if !prefetch_path.exists() {
        return Ok(PrefetchInfo {
            pf_count: 0,
            oldest_time: None,
            newest_time: None,
        });
    }

    let entries = scan_prefetch_directory(&prefetch_path)?;
    let pf_count = count_pf_files(&entries);
    let dates = collect_pf_dates(&entries);
    let (oldest_time, newest_time) = get_oldest_newest_dates(dates);

    Ok(PrefetchInfo {
        pf_count,
        oldest_time,
        newest_time,
    })
}

// === Service Control Manager operations ===

fn open_service_manager() -> Result<SC_HANDLE> {
    unsafe {
        let scm = OpenSCManagerW(PCWSTR::null(), PCWSTR::null(), SC_MANAGER_CONNECT)
            .context("Не удалось открыть Service Control Manager")?;

        if scm.is_invalid() {
            return Err(anyhow::anyhow!("Service Control Manager недоступен"));
        }

        Ok(scm)
    }
}

fn open_sysmain_service(scm: SC_HANDLE, access: u32) -> Result<SC_HANDLE> {
    unsafe {
        let service_name: Vec<u16> = SYSMAIN_SERVICE_NAME.encode_utf16().chain(Some(0)).collect();
        OpenServiceW(scm, PCWSTR(service_name.as_ptr()), access)
            .context("Не удалось открыть службу SysMain")
    }
}

fn query_service_status(service: SC_HANDLE) -> Result<SERVICE_STATUS> {
    unsafe {
        let mut status = SERVICE_STATUS::default();
        QueryServiceStatus(service, &mut status).context("Не удалось получить статус службы")?;
        Ok(status)
    }
}

pub fn get_sysmain_status() -> Result<ServiceStatus> {
    unsafe {
        let scm = match open_service_manager() {
            Ok(h) => h,
            Err(_) => return Ok(ServiceStatus::NotFound),
        };

        let service = match open_sysmain_service(scm, SERVICE_QUERY_STATUS) {
            Ok(s) => s,
            Err(_) => {
                let _ = CloseServiceHandle(scm);
                return Ok(ServiceStatus::NotFound);
            }
        };

        let status = match query_service_status(service) {
            Ok(s) => s,
            Err(_) => {
                let _ = CloseServiceHandle(service);
                let _ = CloseServiceHandle(scm);
                return Ok(ServiceStatus::Unknown);
            }
        };

        let _ = CloseServiceHandle(service);
        let _ = CloseServiceHandle(scm);

        let service_status = match status.dwCurrentState {
            SERVICE_RUNNING => ServiceStatus::Running,
            SERVICE_STOPPED => ServiceStatus::Stopped,
            SERVICE_PAUSED => ServiceStatus::Paused,
            _ => ServiceStatus::Unknown,
        };

        Ok(service_status)
    }
}

fn query_service_config(service: SC_HANDLE) -> Result<QUERY_SERVICE_CONFIGW> {
    unsafe {
        let mut bytes_needed = 0u32;
        let _ = QueryServiceConfigW(service, None, 0, &mut bytes_needed);

        let mut buffer: Vec<u8> = vec![0; bytes_needed as usize];
        let config = buffer.as_mut_ptr() as *mut QUERY_SERVICE_CONFIGW;

        QueryServiceConfigW(service, Some(config), bytes_needed, &mut bytes_needed)
            .context("Не удалось получить конфигурацию службы")?;

        Ok(*config)
    }
}

pub fn get_sysmain_startup_type() -> Result<StartupType> {
    unsafe {
        let scm = match open_service_manager() {
            Ok(h) => h,
            Err(_) => return Ok(StartupType::Unknown),
        };

        let service = match open_sysmain_service(scm, SERVICE_QUERY_CONFIG) {
            Ok(s) => s,
            Err(_) => {
                let _ = CloseServiceHandle(scm);
                return Ok(StartupType::Unknown);
            }
        };

        let config = match query_service_config(service) {
            Ok(c) => c,
            Err(_) => {
                let _ = CloseServiceHandle(service);
                let _ = CloseServiceHandle(scm);
                return Ok(StartupType::Unknown);
            }
        };

        let _ = CloseServiceHandle(service);
        let _ = CloseServiceHandle(scm);

        let startup = match config.dwStartType {
            SERVICE_AUTO_START => StartupType::Automatic,
            SERVICE_DEMAND_START => StartupType::Manual,
            SERVICE_DISABLED => StartupType::Disabled,
            _ => StartupType::Unknown,
        };

        Ok(startup)
    }
}

fn change_service_config(service: SC_HANDLE, start_type: SERVICE_START_TYPE) -> Result<()> {
    unsafe {
        ChangeServiceConfigW(
            service,
            ENUM_SERVICE_TYPE(SERVICE_NO_CHANGE),
            start_type,
            SERVICE_ERROR(SERVICE_NO_CHANGE),
            PCWSTR::null(),
            PCWSTR::null(),
            None,
            PCWSTR::null(),
            PCWSTR::null(),
            PCWSTR::null(),
            PCWSTR::null(),
        )
        .context("Не удалось изменить конфигурацию службы")
    }
}

fn start_service(service: SC_HANDLE) -> Result<()> {
    unsafe {
        let start_result = StartServiceW(service, None);

        if start_result.is_err() {
            let err = windows::core::Error::from_thread();
            if err.code().0 as u32 != ERROR_SERVICE_ALREADY_RUNNING {
                return Err(anyhow::anyhow!("Не удалось запустить службу: {}", err));
            }
        }

        Ok(())
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

        let service = open_sysmain_service(scm, SERVICE_CHANGE_CONFIG | SERVICE_START)
            .context("Не удалось открыть службу SysMain. Требуются права администратора.")?;

        change_service_config(service, SERVICE_AUTO_START)?;
        start_service(service)?;

        let _ = CloseServiceHandle(service);
        let _ = CloseServiceHandle(scm);

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
