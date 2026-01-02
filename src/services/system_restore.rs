//! System Restore service - business logic only.

use crate::domain::{OperationResult, Result, SystemRestoreInfo};
use crate::repositories::registry;
use std::process::Command;

const SR_KEY: &str = r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\SystemRestore";

fn check_registry() -> bool {
    registry::read_hklm(SR_KEY, "RPSessionInterval")
        .map(|v| v == 1)
        .unwrap_or(false)
}

pub fn get_info() -> Result<SystemRestoreInfo> {
    let enabled = check_registry();
    let frequency = registry::read_hklm(SR_KEY, "SystemRestorePointCreationFrequency");
    // DiskPercent is stored under the Cfg subkey
    let disk_percent = registry::read_hklm(
        "SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\SystemRestore\\Cfg",
        "DiskPercent",
    );

    Ok(SystemRestoreInfo {
        enabled,
        method: "Registry",
        frequency_minutes: frequency,
        disk_percent,
    })
}

/// Configure the System Restore auto-creation frequency (minutes).
/// Setting to 0 allows on-demand creation.
pub fn set_frequency(minutes: u32) -> Result<OperationResult> {
    registry::write_hklm(SR_KEY, "SystemRestorePointCreationFrequency", minutes)?;
    Ok(OperationResult::success(format!(
        "Set SystemRestorePointCreationFrequency = {} minutes",
        minutes
    )))
}

/// Configure the disk percentage dedicated to System Restore (integer percent).
pub fn set_disk_percent(percent: u32) -> Result<OperationResult> {
    registry::write_hklm(
        "SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\SystemRestore\\Cfg",
        "DiskPercent",
        percent,
    )?;
    Ok(OperationResult::success(format!(
        "Set DiskPercent = {}%",
        percent
    )))
}

pub fn enable(drive: &str) -> Result<OperationResult> {
    // Registry enable (may require admin) - set RPSessionInterval to 1
    registry::write_hklm(SR_KEY, "RPSessionInterval", 1)?;

    // Also attempt the PowerShell cmdlet to enable the system restore feature
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            &format!("Enable-ComputerRestore -Drive '{}:\\'", drive),
        ])
        .output()?;

    if output.status.success() {
        Ok(OperationResult::success("System Restore enabled"))
    } else {
        // Return success if registry write succeeded
        Ok(OperationResult::success(
            "System Restore enabled via registry",
        ))
    }
}
