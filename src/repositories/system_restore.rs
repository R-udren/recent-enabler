use crate::domain::{AppError, RestoreEventType, RestorePointType, Result};
use serde::{Deserialize, Serialize};
use tracing::{error, info, instrument, trace};
use winreg::enums::*;
use winreg::RegKey;
use wmi::WMIConnection;

#[derive(Deserialize)]
#[allow(non_camel_case_types)]
struct SystemRestore;

#[derive(Serialize)]
struct EnableInput {
    #[serde(rename = "Drive")]
    drive: String,
}

#[derive(Serialize)]
struct CreateRestorePointInput {
    #[serde(rename = "Description")]
    description: String,
    #[serde(rename = "EventType")]
    event_type: u32,
    #[serde(rename = "RestorePointType")]
    restore_point_type: u32,
}

#[derive(Deserialize)]
struct WmiReturnValue {
    #[serde(rename = "ReturnValue")]
    return_value: u32,
}

pub struct SystemRestoreManager {
    wmi_con: WMIConnection,
}

impl SystemRestoreManager {
    /// Initialize WMI connection.
    pub fn new() -> Result<Self> {
        trace!("Initializing SystemRestoreManager");
        // Initialize COM for the current thread
        unsafe {
            let _ = windows::Win32::System::Com::CoInitializeEx(
                None,
                windows::Win32::System::Com::COINIT_MULTITHREADED,
            );
        }

        // Connect to the 'root\default' namespace where SystemRestore lives
        let wmi_con = WMIConnection::with_namespace_path("root\\default").map_err(|e| {
            error!("WMI connection failure to root\\default: {}", e);
            AppError::Other(format!("WMI connection failure: {}", e))
        })?;

        info!("SystemRestoreManager initialized successfully");
        Ok(Self { wmi_con })
    }

    /// Enable System Restore protection on a specific drive (e.g., "C:")
    #[instrument(skip(self))]
    pub fn enable_protection(&self, drive_letter: &str) -> Result<()> {
        let normalized_drive = self.normalize_drive(drive_letter);
        info!("Enabling protection for drive: {}", normalized_drive);

        let input = EnableInput {
            drive: normalized_drive,
        };

        // Execute the static 'Enable' method on the SystemRestore class
        let out: WmiReturnValue = self
            .wmi_con
            .exec_class_method::<SystemRestore, _>("Enable", input)
            .map_err(|e| {
                error!("WMI Enable method failed: {}", e);
                AppError::Other(format!("Failed to enable protection: {}", e))
            })?;

        if out.return_value != 0 {
            error!("WMI Enable returned error code: {}", out.return_value);
            return Err(AppError::Other(format!(
                "WMI Enable returned error code: {}",
                out.return_value
            )));
        }

        info!("Protection enabled successfully");
        Ok(())
    }

    /// Disable System Restore protection on a specific drive
    #[instrument(skip(self))]
    pub fn disable_protection(&self, drive_letter: &str) -> Result<()> {
        let normalized_drive = self.normalize_drive(drive_letter);
        info!("Disabling protection for drive: {}", normalized_drive);

        let input = EnableInput {
            drive: normalized_drive,
        };

        let out: WmiReturnValue = self
            .wmi_con
            .exec_class_method::<SystemRestore, _>("Disable", input)
            .map_err(|e| {
                error!("WMI Disable method failed: {}", e);
                AppError::Other(format!("Failed to disable protection: {}", e))
            })?;

        if out.return_value != 0 {
            error!("WMI Disable returned error code: {}", out.return_value);
            return Err(AppError::Other(format!(
                "WMI Disable returned error code: {}",
                out.return_value
            )));
        }

        info!("Protection disabled successfully");
        Ok(())
    }

    /// Create a new Restore Point
    #[instrument(skip(self))]
    pub fn create_restore_point(
        &self,
        description: &str,
        event_type: RestoreEventType,
        pt_type: RestorePointType,
    ) -> Result<()> {
        info!("Creating restore point: {}", description);
        let input = CreateRestorePointInput {
            description: description.to_owned(),
            event_type: event_type as u32,
            restore_point_type: pt_type as u32,
        };

        let out: WmiReturnValue = self
            .wmi_con
            .exec_class_method::<SystemRestore, _>("CreateRestorePoint", input)
            .map_err(|e| {
                error!("WMI CreateRestorePoint method failed: {}", e);
                AppError::Other(format!("Failed to create restore point: {}", e))
            })?;

        if out.return_value != 0 {
            error!(
                "WMI CreateRestorePoint returned error code: {}",
                out.return_value
            );
            return Err(AppError::Other(format!(
                "WMI CreateRestorePoint returned error code: {}",
                out.return_value
            )));
        }

        info!("Restore point created successfully");
        Ok(())
    }

    /// Check if protection is enabled (Registry Fallback)
    #[instrument(skip(self))]
    pub fn is_protection_enabled(&self, drive_letter: &str) -> Result<bool> {
        let clean_letter = drive_letter.trim_end_matches('\\').trim_end_matches(':');
        trace!("Checking protection status for drive: {}", clean_letter);

        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let path = format!(
            "SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\SystemRestore\\Cfg\\{}",
            clean_letter
        );

        match hklm.open_subkey(path) {
            Ok(_) => {
                trace!("Drive-specific config found, protection is enabled");
                Ok(true)
            }
            Err(_) => {
                trace!("Drive-specific config not found, checking global RPSessionInterval");
                let sr_key = hklm
                    .open_subkey("SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\SystemRestore")
                    .map_err(|e| {
                        error!("Failed to open SystemRestore registry key: {}", e);
                        AppError::Registry(e.to_string())
                    })?;
                let rp_interval: u32 = sr_key.get_value("RPSessionInterval").unwrap_or(0);
                let enabled = rp_interval == 1;
                trace!(
                    "Global RPSessionInterval: {}, enabled: {}",
                    rp_interval,
                    enabled
                );
                Ok(enabled)
            }
        }
    }

    /// Helper to ensure drive is "C:\" format required by WMI
    fn normalize_drive(&self, drive: &str) -> String {
        if !drive.ends_with('\\') {
            format!("{}\\", drive)
        } else {
            drive.to_string()
        }
    }
}
