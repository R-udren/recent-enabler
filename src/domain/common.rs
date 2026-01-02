//! Core domain types - pure data structures with no dependencies.

use std::time::SystemTime;

// =============================================================================
// Operation Results
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

    pub fn failure(message: impl Into<String>) -> Self {
        Self {
            success: false,
            message: message.into(),
            requires_admin: false,
        }
    }

    pub fn requires_admin(mut self) -> Self {
        self.requires_admin = true;
        self
    }
}

// =============================================================================
// File Scanning Results
// =============================================================================

#[derive(Debug, Clone)]
pub struct FileStats {
    pub count: usize,
    pub oldest: Option<SystemTime>,
    pub newest: Option<SystemTime>,
}

impl FileStats {
    pub fn empty() -> Self {
        Self {
            count: 0,
            oldest: None,
            newest: None,
        }
    }
}
