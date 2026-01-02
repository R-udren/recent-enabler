//! Error types for the application.

use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum AppError {
    #[error("Registry error: {0}")]
    Registry(String),

    #[error("Windows service error: {0}")]
    Service(String),

    #[error("File system error: {0}")]
    FileSystem(String),

    #[error("PowerShell error: {0}")]
    PowerShell(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("{0}")]
    Other(String),
}

impl AppError {
    pub fn to_user_string(&self) -> String {
        self.to_string()
    }
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
