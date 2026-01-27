//! Prefetch service - business logic only.

use crate::domain::{OperationResult, PrefetchInfo, Result, ServiceState, StartupMode};
use crate::repositories::{file_system, windows_service};
use std::path::PathBuf;
use windows::Win32::System::Services::*;

const SERVICE_NAME: &str = "SysMain";

fn get_prefetch_path() -> Result<PathBuf> {
    let windows = std::env::var("SystemRoot").or_else(|_| std::env::var("windir"))?;
    Ok(PathBuf::from(windows).join("Prefetch"))
}

fn get_service_info() -> (ServiceState, StartupMode) {
    let scm = match windows_service::open_scm() {
        Ok(s) => s,
        Err(_) => return (ServiceState::Unknown, StartupMode::Unknown),
    };

    let svc = match windows_service::open_service(
        &scm,
        SERVICE_NAME,
        SERVICE_QUERY_STATUS | SERVICE_QUERY_CONFIG,
    ) {
        Ok(s) => s,
        Err(_) => return (ServiceState::Unknown, StartupMode::Unknown),
    };

    let state = windows_service::get_service_state(&svc).unwrap_or(ServiceState::Unknown);
    let mode = windows_service::get_startup_mode(&svc).unwrap_or(StartupMode::Unknown);

    (state, mode)
}

pub fn get_info() -> Result<PrefetchInfo> {
    let path = get_prefetch_path()?;
    let (service_state, startup_mode) = get_service_info();

    let (files, accessible, error) = match file_system::scan_folder(&path, "pf") {
        Ok(f) => (f, true, None),
        Err(_e) => {
            let err_msg = "Requires admin access".to_string();
            (Default::default(), false, Some(err_msg))
        }
    };

    Ok(PrefetchInfo {
        path: path.display().to_string(),
        files,
        accessible,
        error,
        service_state,
        startup_mode,
    })
}

pub fn enable() -> Result<OperationResult> {
    let scm = windows_service::open_scm()?;
    let svc =
        windows_service::open_service(&scm, SERVICE_NAME, SERVICE_CHANGE_CONFIG | SERVICE_START)?;

    windows_service::set_service_auto_start(&svc)?;
    windows_service::start_service(&svc)?;

    Ok(OperationResult::success("Service enabled and started"))
}
