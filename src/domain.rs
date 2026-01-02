//! Domain types and traits for the Recent Enabler application.
//!
//! This module contains all shared types that represent the core domain logic,
//! completely separated from UI concerns. All types here are pure data structures
//! that can be used across different parts of the application.

// Allow dead code for extensibility - these types are designed for future use
#![allow(dead_code)]

use std::time::SystemTime;

// =============================================================================
// Registry Types
// =============================================================================

/// Represents which registry hive(s) a check applies to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegistryLocation {
    /// Current User only (HKCU)
    HKCU,
    /// Local Machine only (HKLM)
    HKLM,
    /// Check applies to both hives
    Both,
}

impl RegistryLocation {
    pub fn as_str(&self) -> &'static str {
        match self {
            RegistryLocation::HKCU => "HKCU",
            RegistryLocation::HKLM => "HKLM",
            RegistryLocation::Both => "HKCU/HKLM",
        }
    }
}

// =============================================================================
// Recent Files Types
// =============================================================================

/// Severity level of a registry check for Recent Files functionality.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CheckSeverity {
    /// Only affects specific UI elements (ShowRecent, ShowFrequent, etc.)
    Minor = 0,
    /// Will break most features (Start_TrackDocs, Start_TrackProgs)
    Important = 1,
    /// Will definitely break functionality (Policy-level blocks)
    Critical = 2,
}

impl CheckSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            CheckSeverity::Minor => "Незначительный",
            CheckSeverity::Important => "Важный",
            CheckSeverity::Critical => "Критический",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            CheckSeverity::Minor => "Влияет только на отображение в UI",
            CheckSeverity::Important => "Влияет на основные функции записи",
            CheckSeverity::Critical => "Полностью блокирует функциональность",
        }
    }
}

/// A single registry check for Recent Files functionality.
#[derive(Debug, Clone)]
pub struct RecentCheck {
    /// Human-readable description of what this check represents
    pub source: String,
    /// Registry location (HKCU, HKLM, or Both)
    pub location: RegistryLocation,
    /// Full registry key path (without hive prefix)
    pub key_path: String,
    /// Value name within the key
    pub value_name: String,
    /// Current value read from registry (None if not present)
    pub current_value: Option<u32>,
    /// Value expected when feature is enabled
    pub expected_for_enabled: u32,
    /// Severity level of this check
    pub severity: CheckSeverity,
    /// Whether this is a Group Policy setting (cannot be overridden by user)
    pub is_policy: bool,
}

impl RecentCheck {
    /// Returns true if the current value matches the expected enabled value.
    pub fn is_enabled(&self) -> bool {
        self.current_value == Some(self.expected_for_enabled)
    }

    /// Returns true if this check is blocking Recent Files functionality.
    pub fn is_blocking(&self) -> bool {
        !self.is_enabled()
    }

    /// Returns true if this is a policy-level block that cannot be overridden.
    pub fn is_policy_block(&self) -> bool {
        self.is_policy && self.is_blocking()
    }
}

/// Result of a single registry check operation.
#[derive(Debug, Clone)]
pub struct RecentCheckResult {
    pub check: RecentCheck,
    pub can_fix: bool,
    pub fix_description: Option<String>,
}

/// Overall status of Recent Files functionality.
#[derive(Debug, Clone, PartialEq)]
pub enum RecentStatus {
    /// All checks passed, Recent Files fully functional
    FullyEnabled,
    /// All checks failed, Recent Files completely disabled
    FullyDisabled,
    /// Some checks passed, some failed
    PartiallyEnabled {
        enabled_features: Vec<String>,
        disabled_features: Vec<String>,
    },
    /// Disabled by Group Policy - user cannot enable without admin/policy change
    DisabledByPolicy { policy_sources: Vec<String> },
}

impl RecentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            RecentStatus::FullyEnabled => "Полностью включена",
            RecentStatus::FullyDisabled => "Полностью отключена",
            RecentStatus::PartiallyEnabled { .. } => "Частично включена",
            RecentStatus::DisabledByPolicy { .. } => "Заблокирована политикой",
        }
    }

    pub fn is_enabled(&self) -> bool {
        matches!(self, RecentStatus::FullyEnabled)
    }

    pub fn is_partially_enabled(&self) -> bool {
        matches!(self, RecentStatus::PartiallyEnabled { .. })
    }

    pub fn is_policy_blocked(&self) -> bool {
        matches!(self, RecentStatus::DisabledByPolicy { .. })
    }
}

/// Complete information about Recent Files folder and status.
#[derive(Debug, Clone)]
pub struct RecentInfo {
    /// Path to the Recent folder
    pub path: String,
    /// Number of .lnk files in the folder
    pub lnk_count: usize,
    /// Oldest file modification time
    pub oldest_time: Option<SystemTime>,
    /// Newest file modification time
    pub newest_time: Option<SystemTime>,
    /// Overall status determined from registry checks
    pub status: RecentStatus,
    /// Individual check results for detailed display
    pub checks: Vec<RecentCheckResult>,
}

// =============================================================================
// Prefetch / SysMain Types
// =============================================================================

/// Status of the Prefetcher based on registry value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrefetcherMode {
    /// Prefetcher is disabled (value = 0)
    Disabled,
    /// Only boot files are prefetched (value = 1)
    BootOnly,
    /// Only application files are prefetched (value = 2)
    ApplicationsOnly,
    /// Both boot and application files (value = 3, default)
    FullyEnabled,
    /// Unknown or unexpected value
    Unknown(u32),
}

impl PrefetcherMode {
    pub fn from_registry_value(value: u32) -> Self {
        match value {
            0 => PrefetcherMode::Disabled,
            1 => PrefetcherMode::BootOnly,
            2 => PrefetcherMode::ApplicationsOnly,
            3 => PrefetcherMode::FullyEnabled,
            v => PrefetcherMode::Unknown(v),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            PrefetcherMode::Disabled => "Отключен",
            PrefetcherMode::BootOnly => "Только загрузка",
            PrefetcherMode::ApplicationsOnly => "Только приложения",
            PrefetcherMode::FullyEnabled => "Полностью включен",
            PrefetcherMode::Unknown(_) => "Неизвестно",
        }
    }

    pub fn is_enabled(&self) -> bool {
        !matches!(self, PrefetcherMode::Disabled)
    }
}

/// Status of the SysMain (Superfetch) service.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceStatus {
    Running,
    Stopped,
    Paused,
    StartPending,
    StopPending,
    Unknown,
    NotFound,
}

impl ServiceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ServiceStatus::Running => "Запущена",
            ServiceStatus::Stopped => "Остановлена",
            ServiceStatus::Paused => "Приостановлена",
            ServiceStatus::StartPending => "Запускается",
            ServiceStatus::StopPending => "Останавливается",
            ServiceStatus::Unknown => "Неизвестно",
            ServiceStatus::NotFound => "Не найдена",
        }
    }

    pub fn is_running(&self) -> bool {
        matches!(self, ServiceStatus::Running)
    }
}

/// Startup type of a Windows service.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartupType {
    Automatic,
    AutomaticDelayed,
    Manual,
    Disabled,
    Unknown,
}

impl StartupType {
    pub fn as_str(&self) -> &'static str {
        match self {
            StartupType::Automatic => "Автоматически",
            StartupType::AutomaticDelayed => "Автоматически (отложенный)",
            StartupType::Manual => "Вручную",
            StartupType::Disabled => "Отключена",
            StartupType::Unknown => "Неизвестно",
        }
    }

    pub fn is_auto(&self) -> bool {
        matches!(self, StartupType::Automatic | StartupType::AutomaticDelayed)
    }
}

/// Information about the Prefetch folder scan.
#[derive(Debug, Clone)]
pub struct PrefetchInfo {
    /// Path to the Prefetch folder
    pub path: String,
    /// Number of .pf files found
    pub pf_count: usize,
    /// Oldest file modification time
    pub oldest_time: Option<SystemTime>,
    /// Newest file modification time
    pub newest_time: Option<SystemTime>,
    /// Whether the folder was accessible
    pub folder_accessible: bool,
    /// Whether admin rights are required to access
    pub requires_admin: bool,
    /// Error message if folder was inaccessible
    pub error_message: Option<String>,
}

/// Combined status of SysMain service and Prefetch functionality.
#[derive(Debug, Clone)]
pub struct SysMainInfo {
    /// Current service status
    pub service_status: ServiceStatus,
    /// Service startup type
    pub startup_type: StartupType,
    /// Prefetcher registry mode
    pub prefetcher_mode: PrefetcherMode,
    /// Superfetch registry mode (similar to prefetcher)
    pub superfetch_mode: PrefetcherMode,
    /// Prefetch folder information
    pub prefetch_info: PrefetchInfo,
}

impl SysMainInfo {
    /// Returns true if the service is properly configured and running.
    pub fn is_fully_enabled(&self) -> bool {
        self.service_status.is_running()
            && self.startup_type.is_auto()
            && self.prefetcher_mode.is_enabled()
    }

    /// Returns a list of issues that need to be fixed.
    pub fn get_issues(&self) -> Vec<String> {
        let mut issues = Vec::new();

        if !self.service_status.is_running() {
            issues.push(format!("Служба SysMain: {}", self.service_status.as_str()));
        }

        if !self.startup_type.is_auto() {
            issues.push(format!("Тип запуска: {}", self.startup_type.as_str()));
        }

        if !self.prefetcher_mode.is_enabled() {
            issues.push(format!("Prefetcher: {}", self.prefetcher_mode.as_str()));
        }

        if !self.prefetch_info.folder_accessible {
            if self.prefetch_info.requires_admin {
                issues.push("Требуются права администратора для доступа к папке".to_string());
            } else if let Some(ref err) = self.prefetch_info.error_message {
                issues.push(format!("Ошибка доступа к папке: {}", err));
            }
        }

        issues
    }
}

// =============================================================================
// System Restore Types
// =============================================================================

/// Method used to interact with System Restore.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemRestoreMethod {
    /// Windows Management Instrumentation
    Wmi,
    /// Native srclient.dll calls
    Native,
    /// PowerShell cmdlets
    PowerShell,
    /// Registry-only detection (no enable capability)
    RegistryOnly,
}

impl SystemRestoreMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            SystemRestoreMethod::Wmi => "WMI",
            SystemRestoreMethod::Native => "Native DLL",
            SystemRestoreMethod::PowerShell => "PowerShell",
            SystemRestoreMethod::RegistryOnly => "Registry",
        }
    }
}

/// Status of System Restore on a specific drive.
#[derive(Debug, Clone)]
pub struct SystemRestoreDriveStatus {
    /// Drive letter (e.g., "C:")
    pub drive: String,
    /// Whether System Restore is enabled for this drive
    pub is_enabled: bool,
    /// Detection method used
    pub detection_method: SystemRestoreMethod,
    /// Error message if detection failed
    pub error_message: Option<String>,
}

/// Overall System Restore information.
#[derive(Debug, Clone)]
pub struct SystemRestoreInfo {
    /// Global System Restore status (service level)
    pub global_enabled: bool,
    /// Per-drive status
    pub drive_statuses: Vec<SystemRestoreDriveStatus>,
    /// Available methods for enabling System Restore
    pub available_methods: Vec<SystemRestoreMethod>,
    /// Preferred method (determined by availability and reliability)
    pub preferred_method: Option<SystemRestoreMethod>,
}

impl SystemRestoreInfo {
    /// Returns the status for a specific drive.
    pub fn get_drive_status(&self, drive: &str) -> Option<&SystemRestoreDriveStatus> {
        let normalized = drive.trim_end_matches('\\').to_uppercase();
        self.drive_statuses
            .iter()
            .find(|s| s.drive.trim_end_matches('\\').to_uppercase() == normalized)
    }

    /// Returns true if System Restore is enabled on the C: drive.
    pub fn is_c_drive_enabled(&self) -> bool {
        self.get_drive_status("C:")
            .map(|s| s.is_enabled)
            .unwrap_or(false)
    }
}

// =============================================================================
// Provider Traits
// =============================================================================

/// Result type alias for domain operations.
pub type DomainResult<T> = anyhow::Result<T>;

/// Trait for System Restore operations.
/// Allows multiple implementations (WMI, Native, PowerShell).
pub trait SystemRestoreProvider: Send + Sync {
    /// Returns the method type this provider uses.
    fn method(&self) -> SystemRestoreMethod;

    /// Check if this provider is available on the current system.
    fn is_available(&self) -> bool;

    /// Check if System Restore is enabled for the given drive.
    fn is_enabled(&self, drive: &str) -> DomainResult<bool>;

    /// Enable System Restore for the given drive.
    fn enable(&self, drive: &str) -> DomainResult<()>;

    /// Disable System Restore for the given drive.
    fn disable(&self, drive: &str) -> DomainResult<()>;
}

// =============================================================================
// Application State Types
// =============================================================================

/// Represents actions that can be performed on system features.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeatureAction {
    Enable,
    Disable,
    Refresh,
}

/// Result of an enable/disable operation.
#[derive(Debug, Clone)]
pub struct OperationResult {
    pub success: bool,
    pub message: String,
    pub details: Option<String>,
    pub requires_restart: bool,
    pub requires_admin: bool,
}

impl OperationResult {
    pub fn success(message: impl Into<String>) -> Self {
        Self {
            success: true,
            message: message.into(),
            details: None,
            requires_restart: false,
            requires_admin: false,
        }
    }

    pub fn failure(message: impl Into<String>) -> Self {
        Self {
            success: false,
            message: message.into(),
            details: None,
            requires_restart: false,
            requires_admin: false,
        }
    }

    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }

    pub fn requires_restart(mut self) -> Self {
        self.requires_restart = true;
        self
    }

    pub fn requires_admin(mut self) -> Self {
        self.requires_admin = true;
        self
    }
}
