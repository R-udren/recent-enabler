use anyhow::{Context, Result};
use std::process::Command;
use winreg::{enums::HKEY_LOCAL_MACHINE, RegKey};

/// Check if System Restore is enabled for C: drive
pub fn is_system_restore_enabled() -> Result<bool> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    if let Ok(system_restore) =
        hklm.open_subkey("SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\SystemRestore")
    {
        let rpsession_interval: Result<u32, _> = system_restore.get_value("RPSessionInterval");
        return Ok(rpsession_interval.unwrap_or(0) == 1);
    }
    Ok(false)
}

/// Enable System Restore on C: drive
pub fn enable_system_restore() -> Result<()> {
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            "Enable-ComputerRestore -Drive 'C:'",
        ])
        .output()
        .context("Failed to execute PowerShell command")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let essential = stderr
            .lines()
            .find(|line| !line.trim().is_empty() && !line.contains("ProgressPreference"))
            .unwrap_or(stderr.as_ref());
        anyhow::bail!("Failed to enable System Restore: {}", essential);
    }

    Ok(())
}

/// Get System Restore status for C: drive
pub fn get_system_restore_info() -> Result<bool> {
    is_system_restore_enabled()
}
