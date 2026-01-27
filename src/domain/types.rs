//! Domain types for the application.

use std::time::SystemTime;
use thiserror::Error;

// =============================================================================
// Errors
// =============================================================================

#[derive(Error, Debug, Clone)]
pub enum AppError {
    #[error("Registry error: {0}")]
    Registry(String),

    #[error("Windows service error: {0}")]
    Service(String),

    #[error("File system error: {0}")]
    FileSystem(String),

    #[error("{0}")]
    Other(String),
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::FileSystem(e.to_string())
    }
}

impl From<std::env::VarError> for AppError {
    fn from(e: std::env::VarError) -> Self {
        AppError::Other(e.to_string())
    }
}

impl From<windows::core::Error> for AppError {
    fn from(e: windows::core::Error) -> Self {
        AppError::Service(e.to_string())
    }
}

pub type Result<T> = std::result::Result<T, AppError>;

// =============================================================================
// Common Types
// =============================================================================

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct OperationResult {
    pub success: bool,
    pub message: String,
    pub requires_admin: bool,
}

impl OperationResult {
    pub fn success(message: impl Into<String>) -> Self {
        Self {
            success: true,
            message: message.into(),
            requires_admin: false,
        }
    }

    pub fn requires_admin(mut self) -> Self {
        self.requires_admin = true;
        self
    }
}

#[derive(Debug, Clone, Default)]
pub struct FileStats {
    pub count: usize,
    pub oldest: Option<SystemTime>,
    pub newest: Option<SystemTime>,
}

// =============================================================================
// Prefetch Types
// =============================================================================

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

// =============================================================================
// Recent Files Types
// =============================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckSeverity {
    Minor,
    Important,
    Critical,
}

#[derive(Debug, Clone)]
pub struct RegistryCheck {
    pub name: String,
    pub key: String,
    pub value: String,
    pub expected: u32,
    pub actual: Option<u32>,
    pub severity: CheckSeverity,
    pub is_policy: bool,
}

impl RegistryCheck {
    pub fn is_ok(&self) -> bool {
        self.actual == Some(self.expected)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RecentStatus {
    FullyEnabled,
    PartiallyEnabled,
    FullyDisabled,
    PolicyBlocked,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct RecentInfo {
    pub path: String,
    pub files: FileStats,
    pub status: RecentStatus,
    pub checks: Vec<RegistryCheck>,
}

// =============================================================================
// System Restore Types
// =============================================================================

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SystemRestoreInfo {
    pub enabled: bool,
    pub method: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub enum RestoreEventType {
    BeginSystemChange = 100,
}

#[derive(Debug, Clone, Copy)]
pub enum RestorePointType {
    ModifySettings = 12,
}
