//! System Restore functionality with multiple provider implementations.
//!
//! This module provides three different methods for interacting with System Restore:
//! 1. PowerShell - Simplest, most compatible but slowest
//! 2. Registry - For detection only (fastest, no admin required)
//! 3. Native DLL - Most performant for enable operations
//!
//! The module uses a provider abstraction to allow fallback between methods.

use crate::domain::{
    DomainResult, OperationResult, SystemRestoreDriveStatus, SystemRestoreInfo,
    SystemRestoreMethod, SystemRestoreProvider,
};
use anyhow::{Context, Result};
use std::process::Command;
use winreg::enums::*;
use winreg::RegKey;

// =============================================================================
// Registry-Based Detection (No Admin Required)
// =============================================================================

const SYSTEM_RESTORE_KEY: &str = r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\SystemRestore";

/// Check if System Restore is globally enabled via registry.
fn is_system_restore_globally_enabled() -> Result<bool> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);

    if let Ok(sr_key) = hklm.open_subkey(SYSTEM_RESTORE_KEY) {
        // Check if System Restore is disabled globally
        let disabled: u32 = sr_key.get_value("DisableSR").unwrap_or(0);
        if disabled == 1 {
            return Ok(false);
        }

        // Check RPSessionInterval - if 1, System Restore is active
        let rp_session: u32 = sr_key.get_value("RPSessionInterval").unwrap_or(0);
        return Ok(rp_session == 1);
    }

    Ok(false)
}

/// Check if System Restore is enabled for a specific drive via registry.
fn is_drive_protected_registry(drive: &str) -> Result<bool> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);

    // Normalize drive letter (remove trailing backslash, ensure colon)
    let drive_normalized = drive.trim_end_matches('\\');
    let drive_letter = drive_normalized
        .chars()
        .next()
        .map(|c| c.to_ascii_uppercase())
        .unwrap_or('C');

    // Check the main System Restore status first
    let global_enabled = is_system_restore_globally_enabled()?;
    if !global_enabled {
        return Ok(false);
    }

    // Check per-volume configuration
    // Path: HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\SystemRestore\Cfg\{DriveLetter}:
    let cfg_path = format!("{}\\Cfg\\{}:", SYSTEM_RESTORE_KEY, drive_letter);

    if let Ok(cfg_key) = hklm.open_subkey(&cfg_path) {
        // If DiskPercent is set and > 0, protection is enabled
        let disk_percent: u32 = cfg_key.get_value("DiskPercent").unwrap_or(0);
        if disk_percent > 0 {
            return Ok(true);
        }
    }

    // Alternative check via VSS (Volume Shadow Copy)
    let vss_path = format!(
        r"SYSTEM\CurrentControlSet\Services\VSS\Diag\SystemRestore\{}:",
        drive_letter
    );
    if hklm.open_subkey(&vss_path).is_ok() {
        return Ok(true);
    }

    // Check WMI style registry
    let sr_config_path = r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\SPP\Clients";
    if let Ok(spp_key) = hklm.open_subkey(sr_config_path) {
        // If this key exists and has subkeys, System Protection might be active
        let _subkey_count = spp_key.enum_keys().count();
        // This is a heuristic - not definitive
    }

    Ok(false)
}

// =============================================================================
// PowerShell Provider
// =============================================================================

/// PowerShell-based System Restore provider.
pub struct PowerShellProvider;

impl PowerShellProvider {
    pub fn new() -> Self {
        Self
    }

    fn run_powershell_command(args: &[&str]) -> Result<String> {
        let output = Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass"])
            .args(args)
            .output()
            .context("Failed to execute PowerShell")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // Filter out noise from PowerShell
            let essential = stderr
                .lines()
                .find(|line| !line.trim().is_empty() && !line.contains("ProgressPreference"))
                .unwrap_or(stderr.as_ref());
            anyhow::bail!("PowerShell error: {}", essential);
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

impl SystemRestoreProvider for PowerShellProvider {
    fn method(&self) -> SystemRestoreMethod {
        SystemRestoreMethod::PowerShell
    }

    fn is_available(&self) -> bool {
        // PowerShell is available on all modern Windows
        Command::new("powershell")
            .args(["-NoProfile", "-Command", "exit 0"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    fn is_enabled(&self, drive: &str) -> DomainResult<bool> {
        let drive_normalized = if drive.ends_with('\\') {
            drive.to_string()
        } else if drive.ends_with(':') {
            format!("{}\\", drive)
        } else {
            format!("{}:\\", drive)
        };

        let script = format!(
            "(Get-ComputerRestorePoint -Drive '{}' -ErrorAction SilentlyContinue) -ne $null -or \
             ((Get-WmiObject -Class Win32_ShadowCopy -ErrorAction SilentlyContinue | \
               Where-Object {{ $_.VolumeName -like '*{}*' }}) -ne $null)",
            drive_normalized,
            drive_normalized.chars().next().unwrap_or('C')
        );

        let output = Self::run_powershell_command(&["-Command", &script])?;
        Ok(output.trim().eq_ignore_ascii_case("true"))
    }

    fn enable(&self, drive: &str) -> DomainResult<()> {
        let drive_normalized = if drive.ends_with('\\') {
            drive.to_string()
        } else if drive.ends_with(':') {
            format!("{}\\", drive)
        } else {
            format!("{}:\\", drive)
        };

        let script = format!("Enable-ComputerRestore -Drive '{}'", drive_normalized);
        Self::run_powershell_command(&["-Command", &script])?;
        Ok(())
    }

    fn disable(&self, drive: &str) -> DomainResult<()> {
        let drive_normalized = if drive.ends_with('\\') {
            drive.to_string()
        } else if drive.ends_with(':') {
            format!("{}\\", drive)
        } else {
            format!("{}:\\", drive)
        };

        let script = format!("Disable-ComputerRestore -Drive '{}'", drive_normalized);
        Self::run_powershell_command(&["-Command", &script])?;
        Ok(())
    }
}

impl Default for PowerShellProvider {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Registry-Only Provider (Detection Only)
// =============================================================================

/// Registry-based System Restore detection (read-only, no admin required).
pub struct RegistryProvider;

impl RegistryProvider {
    pub fn new() -> Self {
        Self
    }
}

impl SystemRestoreProvider for RegistryProvider {
    fn method(&self) -> SystemRestoreMethod {
        SystemRestoreMethod::RegistryOnly
    }

    fn is_available(&self) -> bool {
        // Registry is always available
        true
    }

    fn is_enabled(&self, drive: &str) -> DomainResult<bool> {
        is_drive_protected_registry(drive)}

    fn enable(&self, _drive: &str) -> DomainResult<()> {
        Err(anyhow::anyhow!(
            "Registry provider does not support enable operations. Use PowerShell or Native provider."
        ))
    }

    fn disable(&self, _drive: &str) -> DomainResult<()> {
        Err(anyhow::anyhow!(
            "Registry provider does not support disable operations. Use PowerShell or Native provider."
        ))
    }
}

impl Default for RegistryProvider {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Provider Management
// =============================================================================

/// Get all available System Restore providers in order of preference.
pub fn get_available_providers() -> Vec<Box<dyn SystemRestoreProvider>> {
    let mut providers: Vec<Box<dyn SystemRestoreProvider>> = Vec::new();

    // PowerShell is our primary provider (most reliable for enable operations)
    let ps_provider = PowerShellProvider::new();
    if ps_provider.is_available() {
        providers.push(Box::new(ps_provider));
    }

    // Registry provider for fast detection
    providers.push(Box::new(RegistryProvider::new()));

    providers
}

/// Get the best available provider for enable operations.
pub fn get_enable_provider() -> Option<Box<dyn SystemRestoreProvider>> {
    let providers = get_available_providers();
    providers
        .into_iter()
        .find(|p| p.method() != SystemRestoreMethod::RegistryOnly)
}

/// Get the best available provider for detection.
pub fn get_detection_provider() -> Box<dyn SystemRestoreProvider> {
    // Prefer registry for detection (faster, no admin required)
    Box::new(RegistryProvider::new())
}

// =============================================================================
// High-Level API
// =============================================================================

/// Get comprehensive System Restore information.
pub fn get_system_restore_info() -> DomainResult<SystemRestoreInfo> {
    let global_enabled = is_system_restore_globally_enabled()?;

    // Get status for common drives
    let drives = vec!["C:"];
    let mut drive_statuses = Vec::new();

    let detection_provider = get_detection_provider();

    for drive in drives {
        let is_enabled = detection_provider.is_enabled(drive).unwrap_or(false);
        drive_statuses.push(SystemRestoreDriveStatus {
            drive: drive.to_string(),
            is_enabled,
            detection_method: detection_provider.method(),
            error_message: None,
        });
    }

    // Determine available methods
    let available_methods: Vec<SystemRestoreMethod> = get_available_providers()
        .iter()
        .map(|p| p.method())
        .collect();

    let preferred_method = available_methods
        .iter()
        .find(|m| **m != SystemRestoreMethod::RegistryOnly)
        .copied();

    Ok(SystemRestoreInfo {
        global_enabled,
        drive_statuses,
        available_methods,
        preferred_method,
    })
}

/// Enable System Restore on the specified drive.
pub fn enable_system_restore(drive: &str) -> DomainResult<OperationResult> {
    let provider = match get_enable_provider() {
        Some(p) => p,
        None => {
            return Ok(OperationResult::failure(
                "Нет доступных методов для включения System Restore",
            ));
        }
    };

    match provider.enable(drive) {
        Ok(()) => Ok(OperationResult::success(format!(
            "System Restore успешно включена на диске {} (метод: {})",
            drive,
            provider.method().as_str()
        ))),
        Err(e) => Ok(OperationResult::failure(format!(
            "Не удалось включить System Restore: {}",
            e
        ))
        .requires_admin()),
    }
}

/// Disable System Restore on the specified drive.
#[allow(dead_code)]
pub fn disable_system_restore(drive: &str) -> DomainResult<OperationResult> {
    let provider = match get_enable_provider() {
        Some(p) => p,
        None => {
            return Ok(OperationResult::failure(
                "Нет доступных методов для отключения System Restore",
            ));
        }
    };

    match provider.disable(drive) {
        Ok(()) => Ok(OperationResult::success(format!(
            "System Restore успешно отключена на диске {} (метод: {})",
            drive,
            provider.method().as_str()
        ))),
        Err(e) => Ok(OperationResult::failure(format!(
            "Не удалось отключить System Restore: {}",
            e
        ))
        .requires_admin()),
    }
}

// =============================================================================
// Legacy API Compatibility
// =============================================================================

/// Legacy function: Check if System Restore is enabled for C: drive.
pub fn is_system_restore_enabled() -> Result<bool> {
    let info = get_system_restore_info()?;
    Ok(info.is_c_drive_enabled())
}

/// Legacy function: Get System Restore info as a simple boolean.
pub fn get_system_restore_info_legacy() -> Result<bool> {
    is_system_restore_enabled()
}

/// Legacy function: Enable System Restore on C: drive.
pub fn enable_system_restore_legacy() -> Result<()> {
    let result = enable_system_restore("C:")?;
    if result.success {
        Ok(())
    } else {
        Err(anyhow::anyhow!("{}", result.message))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_powershell_provider_available() {
        let provider = PowerShellProvider::new();
        // PowerShell should be available on Windows
        assert!(provider.is_available());
    }

    #[test]
    fn test_registry_provider_available() {
        let provider = RegistryProvider::new();
        assert!(provider.is_available());
    }

    #[test]
    fn test_get_available_providers() {
        let providers = get_available_providers();
        assert!(!providers.is_empty());
    }
}
