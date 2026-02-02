use crate::{
    error::{RecentEnablerError, Result},
    utils,
};
use std::process::Command;
use winreg::enums::HKEY_LOCAL_MACHINE;

/// Check if System Restore is enabled for C: drive
///
/// # Errors
///
/// Returns error if registry cannot be read
pub fn is_system_restore_enabled() -> Result<bool> {
    let path = r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\SystemRestore";
    Ok(utils::read_reg_dword(HKEY_LOCAL_MACHINE, path, "RPSessionInterval").unwrap_or(0) == 1)
}

/// Enable System Restore on C: drive
///
/// # Errors
///
/// Returns error if `PowerShell` command fails
pub fn enable_system_restore() -> Result {
    #[cfg(windows)]
    use std::os::windows::process::CommandExt;

    #[cfg(windows)]
    const CREATE_NO_WINDOW: u32 = 0x08000000;

    let mut cmd = Command::new("powershell");
    cmd.args([
        "-NoProfile",
        "-WindowStyle",
        "Hidden",
        "-Command",
        "Enable-ComputerRestore -Drive 'C:'",
    ]);

    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);

    let output = cmd.output().map_err(|e| {
        RecentEnablerError::SystemRestoreEnableFailed(format!(
            "Failed to execute PowerShell command: {e}"
        ))
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let essential = stderr
            .lines()
            .find(|line| !line.trim().is_empty() && !line.contains("ProgressPreference"))
            .unwrap_or_else(|| stderr.as_ref());
        return Err(RecentEnablerError::SystemRestoreEnableFailed(
            essential.to_string(),
        ));
    }

    Ok(())
}

/// Get System Restore status for C: drive
///
/// # Errors
///
/// Returns error if status cannot be queried
pub fn get_system_restore_info() -> Result<bool> {
    is_system_restore_enabled()
}
