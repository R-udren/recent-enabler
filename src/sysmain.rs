//! SysMain (Superfetch) service and Prefetch folder management.
//!
//! This module provides comprehensive detection and management of the SysMain service
//! and Prefetch functionality, including:
//! - Single-pass prefetch folder scanning (no admin required for count)
//! - Registry-based prefetcher status detection
//! - Service status and startup type queries
//! - Service enable operations (requires admin)

use crate::domain::{
    DomainResult, OperationResult, PrefetchInfo, PrefetcherMode, ServiceStatus, StartupType,
    SysMainInfo,
};
use anyhow::{Context, Result};
use std::cmp::{max, min};
use std::io::ErrorKind;
use std::path::PathBuf;
use std::time::SystemTime;
use windows::core::PCWSTR;
use windows::Win32::System::Services::*;
use winreg::enums::*;
use winreg::RegKey;

const SYSMAIN_SERVICE_NAME: &str = "SysMain";
const ERROR_SERVICE_ALREADY_RUNNING: u32 = 1056;

// =============================================================================
// Path Operations
// =============================================================================

/// Get the path to the Prefetch folder.
pub fn get_prefetch_folder() -> Result<PathBuf> {
    let windows_dir = std::env::var("SystemRoot")
        .or_else(|_| std::env::var("windir"))
        .context("Failed to get Windows directory path")?;
    Ok(PathBuf::from(windows_dir).join("Prefetch"))
}

// =============================================================================
// Prefetch Folder Scanning (Single Pass, No Admin)
// =============================================================================

/// Scan the Prefetch folder in a single pass without requiring admin rights.
/// This uses streaming to avoid allocating a Vec for all files.
///
/// Returns: (count, oldest_time, newest_time, accessible, requires_admin, error_message)
fn scan_prefetch_directory(
    prefetch_path: &PathBuf,
) -> (
    usize,
    Option<SystemTime>,
    Option<SystemTime>,
    bool,
    bool,
    Option<String>,
) {
    if !prefetch_path.exists() {
        return (0, None, None, false, false, Some("Папка Prefetch не существует".into()));
    }

    let entries = match std::fs::read_dir(prefetch_path) {
        Ok(iter) => iter,
        Err(err) => {
            return match err.kind() {
                ErrorKind::PermissionDenied => (
                    0,
                    None,
                    None,
                    false,
                    true,
                    Some("Нет доступа к папке Prefetch. Запустите с правами администратора.".into()),
                ),
                _ => (
                    0,
                    None,
                    None,
                    false,
                    false,
                    Some(format!("Ошибка чтения папки Prefetch: {}", err)),
                ),
            };
        }
    };

    // Single-pass streaming solution - no Vec allocation
    let (count, oldest, newest) = entries
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext.eq_ignore_ascii_case("pf"))
                .unwrap_or(false)
        })
        .filter_map(|e| e.metadata().ok().and_then(|m| m.modified().ok()))
        .fold(
            (0usize, None::<SystemTime>, None::<SystemTime>),
            |(count, oldest, newest), time| {
                (
                    count + 1,
                    Some(oldest.map_or(time, |old| min(old, time))),
                    Some(newest.map_or(time, |new| max(new, time))),
                )
            },
        );

    (count, oldest, newest, true, false, None)
}

/// Get Prefetch folder information using single-pass scanning.
pub fn get_prefetch_info() -> DomainResult<PrefetchInfo> {
    let prefetch_path = get_prefetch_folder()?;
    let (pf_count, oldest_time, newest_time, folder_accessible, requires_admin, error_message) =
        scan_prefetch_directory(&prefetch_path);

    Ok(PrefetchInfo {
        path: prefetch_path.display().to_string(),
        pf_count,
        oldest_time,
        newest_time,
        folder_accessible,
        requires_admin,
        error_message,
    })
}

// =============================================================================
// Registry-Based Prefetcher Status (No Admin Required)
// =============================================================================

const PREFETCH_PARAMS_KEY: &str =
    r"SYSTEM\CurrentControlSet\Control\Session Manager\Memory Management\PrefetchParameters";

/// Get Prefetcher mode from registry (does not require admin).
pub fn get_prefetcher_mode() -> DomainResult<PrefetcherMode> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);

    let key = match hklm.open_subkey(PREFETCH_PARAMS_KEY) {
        Ok(k) => k,
        Err(_) => return Ok(PrefetcherMode::Unknown(0)),
    };

    let enable_prefetch: u32 = key.get_value("EnablePrefetcher").unwrap_or(3);
    Ok(PrefetcherMode::from_registry_value(enable_prefetch))
}

/// Get Superfetch mode from registry (does not require admin).
pub fn get_superfetch_mode() -> DomainResult<PrefetcherMode> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);

    let key = match hklm.open_subkey(PREFETCH_PARAMS_KEY) {
        Ok(k) => k,
        Err(_) => return Ok(PrefetcherMode::Unknown(0)),
    };

    let enable_superfetch: u32 = key.get_value("EnableSuperfetch").unwrap_or(3);
    Ok(PrefetcherMode::from_registry_value(enable_superfetch))
}

// =============================================================================
// Service Control Manager Operations
// =============================================================================

fn open_service_manager() -> Result<SC_HANDLE> {
    unsafe {
        let scm = OpenSCManagerW(PCWSTR::null(), PCWSTR::null(), SC_MANAGER_CONNECT)
            .context("Failed to open Service Control Manager")?;

        if scm.is_invalid() {
            return Err(anyhow::anyhow!("Service Control Manager unavailable"));
        }

        Ok(scm)
    }
}

fn open_sysmain_service(scm: SC_HANDLE, access: u32) -> Result<SC_HANDLE> {
    unsafe {
        let service_name: Vec<u16> = SYSMAIN_SERVICE_NAME.encode_utf16().chain(Some(0)).collect();
        OpenServiceW(scm, PCWSTR(service_name.as_ptr()), access)
            .context("Failed to open SysMain service")
    }
}

fn query_service_status(service: SC_HANDLE) -> Result<SERVICE_STATUS> {
    unsafe {
        let mut status = SERVICE_STATUS::default();
        QueryServiceStatus(service, &mut status).context("Failed to get service status")?;
        Ok(status)
    }
}

/// Get SysMain service status.
pub fn get_sysmain_status() -> DomainResult<ServiceStatus> {
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
            SERVICE_START_PENDING => ServiceStatus::StartPending,
            SERVICE_STOP_PENDING => ServiceStatus::StopPending,
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
            .context("Failed to query service config")?;

        Ok(*config)
    }
}

/// Get SysMain service startup type.
pub fn get_sysmain_startup_type() -> DomainResult<StartupType> {
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

// =============================================================================
// Service Enable Operations (Requires Admin)
// =============================================================================

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
        .context("Failed to change service configuration")
    }
}

fn start_service(service: SC_HANDLE) -> Result<()> {
    unsafe {
        let start_result = StartServiceW(service, None);

        if start_result.is_err() {
            let err = windows::core::Error::from_thread();
            if err.code().0 as u32 != ERROR_SERVICE_ALREADY_RUNNING {
                return Err(anyhow::anyhow!("Failed to start service: {}", err));
            }
        }

        Ok(())
    }
}

/// Enable SysMain service (set to auto-start and start it).
/// Requires administrator privileges.
pub fn enable_sysmain() -> DomainResult<OperationResult> {
    unsafe {
        let scm = match OpenSCManagerW(PCWSTR::null(), PCWSTR::null(), SC_MANAGER_ALL_ACCESS) {
            Ok(h) if !h.is_invalid() => h,
            _ => {
                return Ok(OperationResult::failure(
                    "Не удалось открыть Service Control Manager. Требуются права администратора.",
                )
                .requires_admin());
            }
        };

        let service =
            match open_sysmain_service(scm, SERVICE_CHANGE_CONFIG | SERVICE_START) {
                Ok(s) => s,
                Err(_) => {
                    let _ = CloseServiceHandle(scm);
                    return Ok(OperationResult::failure(
                        "Не удалось открыть службу SysMain. Требуются права администратора.",
                    )
                    .requires_admin());
                }
            };

        // Set to auto-start
        if let Err(e) = change_service_config(service, SERVICE_AUTO_START) {
            let _ = CloseServiceHandle(service);
            let _ = CloseServiceHandle(scm);
            return Ok(OperationResult::failure(format!(
                "Не удалось изменить тип запуска службы: {}",
                e
            )));
        }

        // Start the service
        if let Err(e) = start_service(service) {
            let _ = CloseServiceHandle(service);
            let _ = CloseServiceHandle(scm);
            return Ok(OperationResult::failure(format!(
                "Не удалось запустить службу: {}",
                e
            )));
        }

        let _ = CloseServiceHandle(service);
        let _ = CloseServiceHandle(scm);

        Ok(OperationResult::success(
            "Служба SysMain успешно включена и запущена!",
        ))
    }
}

// =============================================================================
// Combined Status Information
// =============================================================================

/// Get complete SysMain and Prefetch information.
pub fn get_sysmain_info() -> DomainResult<SysMainInfo> {
    let service_status = get_sysmain_status()?;
    let startup_type = get_sysmain_startup_type()?;
    let prefetcher_mode = get_prefetcher_mode()?;
    let superfetch_mode = get_superfetch_mode()?;
    let prefetch_info = get_prefetch_info()?;

    Ok(SysMainInfo {
        service_status,
        startup_type,
        prefetcher_mode,
        superfetch_mode,
        prefetch_info,
    })
}

// =============================================================================
// Legacy API Compatibility
// =============================================================================

/// Legacy struct for backwards compatibility.
pub struct LegacyPrefetchInfo {
    pub pf_count: usize,
    pub oldest_time: Option<SystemTime>,
    pub newest_time: Option<SystemTime>,
}

/// Legacy function: Get prefetch info in old format.
pub fn get_legacy_prefetch_info() -> Result<LegacyPrefetchInfo> {
    let info = get_prefetch_info()?;
    
    if !info.folder_accessible {
        if info.requires_admin {
            return Err(anyhow::anyhow!(
                "Нет доступа к папке Prefetch. Запустите программу с правами администратора."
            ));
        } else if let Some(err) = info.error_message {
            return Err(anyhow::anyhow!("{}", err));
        }
    }

    Ok(LegacyPrefetchInfo {
        pf_count: info.pf_count,
        oldest_time: info.oldest_time,
        newest_time: info.newest_time,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_prefetch_folder() {
        let path = get_prefetch_folder().unwrap();
        assert!(path.to_string_lossy().contains("Prefetch"));
    }

    #[test]
    fn test_prefetcher_mode_from_value() {
        assert_eq!(
            PrefetcherMode::from_registry_value(0),
            PrefetcherMode::Disabled
        );
        assert_eq!(
            PrefetcherMode::from_registry_value(1),
            PrefetcherMode::BootOnly
        );
        assert_eq!(
            PrefetcherMode::from_registry_value(2),
            PrefetcherMode::ApplicationsOnly
        );
        assert_eq!(
            PrefetcherMode::from_registry_value(3),
            PrefetcherMode::FullyEnabled
        );
    }
}
