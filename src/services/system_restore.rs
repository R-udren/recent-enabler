//! System Restore service - business logic only.

use crate::domain::{
    OperationResult, RestoreEventType, RestorePointType, Result, SystemRestoreInfo,
};
use crate::repositories::registry;
use crate::repositories::system_restore::SystemRestoreManager;
use tracing::{info, warn};

const SR_KEY: &str = r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\SystemRestore";
const DEFAULT_FREQUENCY_MINUTES: u32 = 1440; // 24 hours

pub fn get_info() -> Result<SystemRestoreInfo> {
    info!("Fetching System Restore info");
    let manager = SystemRestoreManager::new()?;
    let enabled = manager.is_protection_enabled("C:")?;

    Ok(SystemRestoreInfo {
        enabled,
        method: "WMI/Registry",
    })
}

pub fn enable(drive: &str) -> Result<OperationResult> {
    info!("Enabling System Restore for drive: {}", drive);
    let manager = SystemRestoreManager::new()?;

    // Enable protection
    manager.enable_protection(drive)?;

    // Ensure default frequency exists (24 hours)
    if registry::read_hklm(SR_KEY, "SystemRestorePointCreationFrequency").is_none() {
        info!("Setting default restore point creation frequency");
        let _ = registry::write_hklm(
            SR_KEY,
            "SystemRestorePointCreationFrequency",
            DEFAULT_FREQUENCY_MINUTES,
        );
    }

    // Create a restore point
    info!("Attempting to create initial restore point");
    if let Err(e) = manager.create_restore_point(
        "Recent Enabler Auto Restore Point",
        RestoreEventType::BeginSystemChange,
        RestorePointType::ModifySettings,
    ) {
        warn!(
            "Failed to create initial restore point: {}. System Restore is still enabled.",
            e
        );
        return Ok(OperationResult::success(
            "System Restore enabled, but failed to create restore point",
        ));
    }

    Ok(OperationResult::success(
        "System Restore enabled and restore point created",
    ))
}
