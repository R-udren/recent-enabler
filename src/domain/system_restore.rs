//! System Restore domain types.

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SystemRestoreInfo {
    pub enabled: bool,
    pub method: &'static str,
    pub frequency_minutes: Option<u32>,
    pub disk_percent: Option<u32>,
}
