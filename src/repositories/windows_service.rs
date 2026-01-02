//! Windows Service Control Manager helpers.

use crate::domain::{Result, ServiceState, StartupMode};
use windows::core::PCWSTR;
use windows::Win32::System::Services::*;

pub struct ServiceHandle(SC_HANDLE);

impl Drop for ServiceHandle {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseServiceHandle(self.0);
        }
    }
}

pub fn open_scm() -> Result<ServiceHandle> {
    unsafe {
        let scm = OpenSCManagerW(PCWSTR::null(), PCWSTR::null(), SC_MANAGER_CONNECT)
            .map_err(|e| crate::domain::AppError::Service(format!("Failed to open SCM: {}", e)))?;
        Ok(ServiceHandle(scm))
    }
}

pub fn open_service(scm: &ServiceHandle, name: &str, access: u32) -> Result<ServiceHandle> {
    unsafe {
        let name_wide: Vec<u16> = name.encode_utf16().chain(Some(0)).collect();
        let svc = OpenServiceW(scm.0, PCWSTR(name_wide.as_ptr()), access)
            .map_err(|e| crate::domain::AppError::Service(format!("Failed to open service {}: {}", name, e)))?;
        Ok(ServiceHandle(svc))
    }
}

pub fn get_service_state(svc: &ServiceHandle) -> Result<ServiceState> {
    unsafe {
        let mut status = SERVICE_STATUS::default();
        QueryServiceStatus(svc.0, &mut status)?;
        Ok(match status.dwCurrentState {
            SERVICE_RUNNING => ServiceState::Running,
            SERVICE_STOPPED => ServiceState::Stopped,
            _ => ServiceState::Unknown,
        })
    }
}

pub fn get_startup_mode(svc: &ServiceHandle) -> Result<StartupMode> {
    unsafe {
        let mut bytes_needed = 0u32;
        let _ = QueryServiceConfigW(svc.0, None, 0, &mut bytes_needed);

        let mut buffer: Vec<u8> = vec![0; bytes_needed as usize];
        let config = buffer.as_mut_ptr() as *mut QUERY_SERVICE_CONFIGW;

        QueryServiceConfigW(svc.0, Some(config), bytes_needed, &mut bytes_needed)?;

        Ok(match (*config).dwStartType {
            SERVICE_AUTO_START => StartupMode::Automatic,
            SERVICE_DEMAND_START => StartupMode::Manual,
            SERVICE_DISABLED => StartupMode::Disabled,
            _ => StartupMode::Unknown,
        })
    }
}

pub fn set_service_auto_start(svc: &ServiceHandle) -> Result<()> {
    unsafe {
        ChangeServiceConfigW(
            svc.0,
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
        )?;
        Ok(())
    }
}

pub fn start_service(svc: &ServiceHandle) -> Result<()> {
    unsafe {
        let _ = StartServiceW(svc.0, None);
        Ok(())
    }
}
