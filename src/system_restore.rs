use crate::{
    error::{RecentEnablerError, Result},
    utils,
};
use std::process::Command;
use winreg::enums::HKEY_LOCAL_MACHINE;

/// Check if System Restore is enabled for C: drive
pub fn is_system_restore_enabled() -> Result<bool> {
    let path = r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\SystemRestore";
    Ok(utils::read_reg_dword(HKEY_LOCAL_MACHINE, path, "RPSessionInterval").unwrap_or(0) == 1)
}

/// Enable System Restore on C: drive
pub fn enable_system_restore() -> Result {
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            "Enable-ComputerRestore -Drive 'C:'",
        ])
        .output()
        .map_err(|e| {
            RecentEnablerError::SystemRestoreEnableFailed(format!(
                "Failed to execute PowerShell command: {}",
                e
            ))
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let essential = stderr
            .lines()
            .find(|line| !line.trim().is_empty() && !line.contains("ProgressPreference"))
            .unwrap_or(stderr.as_ref());
        return Err(RecentEnablerError::SystemRestoreEnableFailed(
            essential.to_string(),
        ));
    }

    Ok(())
}

/// Get System Restore status for C: drive
pub fn get_system_restore_info() -> Result<bool> {
    is_system_restore_enabled()
}
