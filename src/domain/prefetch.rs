//! Prefetch domain types.

use super::common::FileStats;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceState {
    Running,
    Stopped,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartupMode {
    Automatic,
    Manual,
    Disabled,
    Unknown,
}

impl StartupMode {
    pub fn is_auto(&self) -> bool {
        matches!(self, Self::Automatic)
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PrefetchInfo {
    pub path: String,
    pub files: FileStats,
    pub accessible: bool,
    pub error: Option<String>,
    pub service_state: ServiceState,
    pub startup_mode: StartupMode,
}

impl PrefetchInfo {
    pub fn is_ok(&self) -> bool {
        self.service_state == ServiceState::Running && self.startup_mode.is_auto()
    }
}
