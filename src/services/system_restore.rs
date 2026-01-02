//! System Restore service - business logic only.

use crate::domain::{AppError, OperationResult, Result, SystemRestoreInfo};
use crate::repositories::registry;
use std::process::Command;

const SR_KEY: &str = r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\SystemRestore";
const DEFAULT_FREQUENCY_MINUTES: u32 = 1440; // 24 hours

fn check_registry() -> bool {
    registry::read_hklm(SR_KEY, "RPSessionInterval")
        .map(|v| v == 1)
        .unwrap_or(false)
}

fn enable_via_wmi(drive: &str) -> Result<()> {
    // Ensure drive format like `C:\`
    let drive_path = if drive.ends_with('\\') {
        drive.to_string()
    } else {
        format!("{}\\", drive)
    };

    // Use PowerShell's Invoke-WmiMethod to run the WMI Enable method reliably
    let cmd = format!(
        "Invoke-WmiMethod -Namespace 'root\\default' -Class SystemRestore -Name Enable -ArgumentList '{}'",
        drive_path
    );

    let status = Command::new("powershell")
        .arg("-NoProfile")
        .arg("-Command")
        .arg(cmd)
        .status()
        .map_err(|e| AppError::Other(e.to_string()))?;

    if status.success() {
        Ok(())
    } else {
        Err(AppError::Other(format!(
            "WMI enable failed: status {}",
            status
        )))
    }
}

pub fn get_info() -> Result<SystemRestoreInfo> {
    let enabled = check_registry();

    Ok(SystemRestoreInfo {
        enabled,
        method: "Registry",
    })
}

pub fn enable(drive: &str) -> Result<OperationResult> {
    // Try WMI approach first
    if enable_via_wmi(drive).is_ok() {
        // Ensure default frequency exists (24 hours)
        if registry::read_hklm(SR_KEY, "SystemRestorePointCreationFrequency").is_none() {
            let _ = registry::write_hklm(
                SR_KEY,
                "SystemRestorePointCreationFrequency",
                DEFAULT_FREQUENCY_MINUTES,
            );
        }
        return Ok(OperationResult::success("System Restore enabled via WMI"));
    }

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
        // Ensure default frequency exists
        if registry::read_hklm(SR_KEY, "SystemRestorePointCreationFrequency").is_none() {
            let _ = registry::write_hklm(
                SR_KEY,
                "SystemRestorePointCreationFrequency",
                DEFAULT_FREQUENCY_MINUTES,
            );
        }
        Ok(OperationResult::success("System Restore enabled"))
    } else {
        // Return success if registry write succeeded
        Ok(OperationResult::success(
            "System Restore enabled via registry",
        ))
    }
}
