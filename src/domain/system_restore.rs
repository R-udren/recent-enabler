//! System Restore domain types.

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SystemRestoreInfo {
    pub enabled: bool,
    pub method: &'static str,
}

/// System Restore Event Types
#[derive(Debug, Clone, Copy)]
pub enum RestoreEventType {
    BeginSystemChange = 100,
    EndSystemChange = 101,
}

/// System Restore Point Types
#[derive(Debug, Clone, Copy)]
pub enum RestorePointType {
    ApplicationInstall = 0,
    ApplicationUninstall = 1,
    DeviceDriverInstall = 10,
    ModifySettings = 12,
    CancelledOperation = 13,
}
