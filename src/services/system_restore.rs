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
    Ok(SystemRestoreInfo {
        enabled,
        method: "Registry",
    })
}

pub fn enable(drive: &str) -> Result<OperationResult> {
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            &format!("Enable-ComputerRestore -Drive '{}:\'", drive),
        ])
        .output()?;

    if output.status.success() {
        Ok(OperationResult::success("System Restore enabled"))
    } else {
        Ok(OperationResult::failure("Failed to enable").requires_admin())
    }
}
