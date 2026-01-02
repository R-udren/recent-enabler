//! Recent Files domain types.

use super::common::FileStats;

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
