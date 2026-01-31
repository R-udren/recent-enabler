use crate::{
    error::{RecentEnablerError, Result},
    utils,
};
use std::path::PathBuf;
use windows::core::PCWSTR;
use windows::Win32::System::Services::{
    ChangeServiceConfigW, CloseServiceHandle, OpenSCManagerW, OpenServiceW, QueryServiceConfigW,
    QueryServiceStatus, StartServiceW, ENUM_SERVICE_TYPE, QUERY_SERVICE_CONFIGW, SC_HANDLE,
    SC_MANAGER_ALL_ACCESS, SC_MANAGER_CONNECT, SERVICE_AUTO_START, SERVICE_CHANGE_CONFIG,
    SERVICE_DEMAND_START, SERVICE_DISABLED, SERVICE_ERROR, SERVICE_NO_CHANGE, SERVICE_PAUSED,
    SERVICE_QUERY_CONFIG, SERVICE_QUERY_STATUS, SERVICE_RUNNING, SERVICE_START, SERVICE_STATUS,
    SERVICE_STOPPED,
};

const SYSMAIN_SERVICE_NAME: &str = "SysMain";
const ERROR_SERVICE_ALREADY_RUNNING: u32 = 1056;

pub struct PrefetchInfo {
    pub pf_count: usize,
    pub oldest_time: Option<std::time::SystemTime>,
    pub newest_time: Option<std::time::SystemTime>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServiceStatus {
    Running,
    Stopped,
    Paused,
    Unknown,
    NotFound,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StartupType {
    Automatic,
    Manual,
    Disabled,
    Unknown,
}

impl StartupType {
    #[must_use]
    pub const fn as_str(&self) -> &str {
        match self {
            Self::Automatic => "Автоматически",
            Self::Manual => "Вручную",
            Self::Disabled => "Отключена",
            Self::Unknown => "Неизвестно",
        }
    }
}

// === Path and folder operations ===

/// Get the path to the Prefetch folder
///
/// # Errors
///
/// Returns error if SystemRoot/windir environment variables are not set
pub fn get_prefetch_folder() -> Result<PathBuf> {
    let windows_dir = std::env::var("SystemRoot")
        .or_else(|_| std::env::var("windir"))
        .map_err(|e| {
            RecentEnablerError::WindowsPathNotFound(format!("SystemRoot/windir not found: {e}"))
        })?;
    Ok(PathBuf::from(windows_dir).join("Prefetch"))
}

/// Get statistics about files in Prefetch folder
///
/// # Errors
///
/// Returns error if folder doesn't exist or cannot be read
pub fn get_prefetch_info() -> Result<PrefetchInfo> {
    let prefetch_path = get_prefetch_folder()?;
    let stats = utils::get_directory_stats(&prefetch_path, "pf")
        .map_err(|e| RecentEnablerError::PrefetchInfoFailed(e.to_string()))?;

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
        let scm = OpenSCManagerW(PCWSTR::null(), PCWSTR::null(), access).map_err(|e| {
            RecentEnablerError::ServiceManagerOpenFailed(format!("OpenSCManagerW failed: {e}"))
        })?;
        let scm_handle = ServiceHandle(scm);

        let service_name: Vec<u16> = SYSMAIN_SERVICE_NAME.encode_utf16().chain(Some(0)).collect();
        let service = OpenServiceW(scm_handle.0, PCWSTR(service_name.as_ptr()), service_access)
            .map_err(|e| {
                RecentEnablerError::SysMainServiceNotFound(format!("OpenServiceW failed: {e}"))
            })?;
        let service_handle = ServiceHandle(service);

        f(service_handle.0)
    }
}

/// Get `SysMain` service status
///
/// # Errors
///
/// Returns error if service cannot be queried
pub fn get_sysmain_status() -> Result<ServiceStatus> {
    let res = with_service(SC_MANAGER_CONNECT, SERVICE_QUERY_STATUS, |service| unsafe {
        let mut status = SERVICE_STATUS::default();
        QueryServiceStatus(service, &raw mut status).map_err(|e| {
            RecentEnablerError::SysMainStatusQueryFailed(format!("QueryServiceStatus failed: {e}"))
        })?;

        Ok(match status.dwCurrentState {
            SERVICE_RUNNING => ServiceStatus::Running,
            SERVICE_STOPPED => ServiceStatus::Stopped,
            SERVICE_PAUSED => ServiceStatus::Paused,
            _ => ServiceStatus::Unknown,
        })
    });

    res.or(Ok(ServiceStatus::NotFound))
}

/// Get `SysMain` service startup type
///
/// # Errors
///
/// Returns error if service configuration cannot be queried
pub fn get_sysmain_startup_type() -> Result<StartupType> {
    let res = with_service(SC_MANAGER_CONNECT, SERVICE_QUERY_CONFIG, |service| unsafe {
        let mut bytes_needed = 0u32;
        let _ = QueryServiceConfigW(service, None, 0, &raw mut bytes_needed);

        let mut buffer: Vec<u8> = vec![0; bytes_needed as usize];
        #[allow(clippy::cast_ptr_alignment)]
        let config = buffer.as_mut_ptr().cast::<QUERY_SERVICE_CONFIGW>();

        QueryServiceConfigW(service, Some(config), bytes_needed, &raw mut bytes_needed).map_err(
            |e| {
                RecentEnablerError::SysMainConfigQueryFailed(format!(
                    "QueryServiceConfigW failed: {e}"
                ))
            },
        )?;

        Ok(match (*config).dwStartType {
            SERVICE_AUTO_START => StartupType::Automatic,
            SERVICE_DEMAND_START => StartupType::Manual,
            SERVICE_DISABLED => StartupType::Disabled,
            _ => StartupType::Unknown,
        })
    });

    res.or(Ok(StartupType::Unknown))
}

/// Enable and start `SysMain` service
///
/// # Errors
///
/// Returns error if service cannot be configured or started
pub fn enable_sysmain() -> Result {
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
            .map_err(|e| {
                RecentEnablerError::SysMainEnableFailed(format!("ChangeServiceConfigW failed: {e}"))
            })?;

            let start_result = StartServiceW(service, None);
            if start_result.is_err() {
                let err = windows::core::Error::from_thread();
                if err.code().0.cast_unsigned() != ERROR_SERVICE_ALREADY_RUNNING {
                    return Err(RecentEnablerError::SysMainEnableFailed(format!(
                        "StartServiceW failed: {err}"
                    )));
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
