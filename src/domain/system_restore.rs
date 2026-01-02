//! System Restore domain types.

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SystemRestoreInfo {
    pub enabled: bool,
    pub method: &'static str,
}
