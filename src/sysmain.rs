use crate::utils;
use anyhow::{Context, Result};
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

pub fn get_prefetch_info() -> Result<PrefetchInfo> {
    let prefetch_path = get_prefetch_folder()?;
    let stats = utils::get_directory_stats(&prefetch_path, "pf")?;

    Ok(PrefetchInfo {
        pf_count: stats.count,
        oldest_time: stats.oldest,
        newest_time: stats.newest,
    })
}

// === Service Control Manager operations ===

struct ServiceHandle(SC_HANDLE);

impl Drop for ServiceHandle {
    fn drop(&mut self) {
        if !self.0.is_invalid() {
            unsafe {
                let _ = CloseServiceHandle(self.0);
            }
        }
    }
}

fn with_service<F, R>(access: u32, service_access: u32, f: F) -> Result<R>
where
    F: FnOnce(SC_HANDLE) -> Result<R>,
{
    unsafe {
        let scm = OpenSCManagerW(PCWSTR::null(), PCWSTR::null(), access)
            .context("Не удалось открыть Service Control Manager")?;
        let scm = ServiceHandle(scm);

        let service_name: Vec<u16> = SYSMAIN_SERVICE_NAME.encode_utf16().chain(Some(0)).collect();
        let service = OpenServiceW(scm.0, PCWSTR(service_name.as_ptr()), service_access)
            .context("Не удалось открыть службу SysMain")?;
        let service = ServiceHandle(service);

        f(service.0)
    }
}

pub fn get_sysmain_status() -> Result<ServiceStatus> {
    let res = with_service(SC_MANAGER_CONNECT, SERVICE_QUERY_STATUS, |service| unsafe {
        let mut status = SERVICE_STATUS::default();
        QueryServiceStatus(service, &mut status).context("Не удалось получить статус службы")?;

        Ok(match status.dwCurrentState {
            SERVICE_RUNNING => ServiceStatus::Running,
            SERVICE_STOPPED => ServiceStatus::Stopped,
            SERVICE_PAUSED => ServiceStatus::Paused,
            _ => ServiceStatus::Unknown,
        })
    });

    match res {
        Ok(s) => Ok(s),
        Err(_) => Ok(ServiceStatus::NotFound),
    }
}

pub fn get_sysmain_startup_type() -> Result<StartupType> {
    let res = with_service(SC_MANAGER_CONNECT, SERVICE_QUERY_CONFIG, |service| unsafe {
        let mut bytes_needed = 0u32;
        let _ = QueryServiceConfigW(service, None, 0, &mut bytes_needed);

        let mut buffer: Vec<u8> = vec![0; bytes_needed as usize];
        let config = buffer.as_mut_ptr() as *mut QUERY_SERVICE_CONFIGW;

        QueryServiceConfigW(service, Some(config), bytes_needed, &mut bytes_needed)
            .context("Не удалось получить конфигурацию службы")?;

        Ok(match (*config).dwStartType {
            SERVICE_AUTO_START => StartupType::Automatic,
            SERVICE_DEMAND_START => StartupType::Manual,
            SERVICE_DISABLED => StartupType::Disabled,
            _ => StartupType::Unknown,
        })
    });

    match res {
        Ok(s) => Ok(s),
        Err(_) => Ok(StartupType::Unknown),
    }
}

pub fn enable_sysmain() -> Result<()> {
    with_service(
        SC_MANAGER_ALL_ACCESS,
        SERVICE_CHANGE_CONFIG | SERVICE_START,
        |service| unsafe {
            ChangeServiceConfigW(
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
            )
            .context("Не удалось изменить конфигурацию службы")?;

            let start_result = StartServiceW(service, None);
            if start_result.is_err() {
                let err = windows::core::Error::from_thread();
                if err.code().0 as u32 != ERROR_SERVICE_ALREADY_RUNNING {
                    return Err(anyhow::anyhow!("Не удалось запустить службу: {}", err));
                }
            }
            Ok(())
        },
    )
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
