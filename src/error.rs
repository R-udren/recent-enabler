use thiserror::Error;

pub type Result<T = (), E = RecentEnablerError> = std::result::Result<T, E>;

#[derive(Error, Debug, Clone)]
pub enum RecentEnablerError {
    #[error("Failed to get Recent folder path: {0}")]
    RecentFolderNotFound(String),

    #[error("Failed to read Recent folder statistics: {0}")]
    RecentInfoFailed(String),

    #[error("Failed to check Recent registry settings: {0}")]
    RecentRegistryReadFailed(String),

    #[error("Failed to enable Recent: {0}")]
    RecentEnableFailed(String),

    #[error("Recent is already enabled")]
    RecentAlreadyEnabled,

    #[error("Failed to get Prefetch folder path: {0}")]
    PrefetchFolderNotFound(String),

    #[error("Failed to read Prefetch folder statistics: {0}")]
    PrefetchInfoFailed(String),

    #[error("Failed to open Service Control Manager: {0}")]
    ServiceManagerOpenFailed(String),

    #[error("Failed to open SysMain service: {0}")]
    SysMainServiceNotFound(String),

    #[error("Failed to query SysMain service status: {0}")]
    SysMainStatusQueryFailed(String),

    #[error("Failed to query SysMain service configuration: {0}")]
    SysMainConfigQueryFailed(String),

    #[error("Failed to enable SysMain service: {0}")]
    SysMainEnableFailed(String),

    #[error("SysMain service is already running and set to automatic")]
    SysMainAlreadyEnabled,

    #[error("Administrator privileges required to enable SysMain service")]
    SysMainRequiresAdmin,

    #[error("Failed to check System Restore status: {0}")]
    SystemRestoreCheckFailed(String),

    #[error("Failed to enable System Restore: {0}")]
    SystemRestoreEnableFailed(String),

    #[error("Administrator privileges required to enable System Restore")]
    SystemRestoreRequiresAdmin,

    #[error("System Restore is already enabled")]
    SystemRestoreAlreadyEnabled,

    #[error("Failed to get Windows system path: {0}")]
    WindowsPathNotFound(String),

    #[error("Failed to read directory: {0}")]
    DirectoryReadFailed(String),

    #[error("Failed to read registry value: {0}")]
    RegistryReadFailed(String),

    #[error("Failed to write registry value: {0}")]
    RegistryWriteFailed(String),
}

impl RecentEnablerError {
    /// Translate error to Russian for UI display
    pub fn to_russian(&self) -> String {
        match self {
            Self::RecentFolderNotFound(e) => format!("Не удалось найти папку Recent: {}", e),
            Self::RecentInfoFailed(e) => format!("Не удалось прочитать статистику Recent: {}", e),
            Self::RecentRegistryReadFailed(e) => {
                format!("Не удалось прочитать настройки реестра Recent: {}", e)
            }
            Self::RecentEnableFailed(e) => format!("Не удалось включить Recent: {}", e),
            Self::RecentAlreadyEnabled => "Запись в Recent уже включена".to_string(),
            Self::PrefetchFolderNotFound(e) => format!("Не удалось найти папку Prefetch: {}", e),
            Self::PrefetchInfoFailed(e) => {
                format!("Не удалось прочитать статистику Prefetch: {}", e)
            }
            Self::ServiceManagerOpenFailed(e) => {
                format!("Не удалось открыть Service Control Manager: {}", e)
            }
            Self::SysMainServiceNotFound(e) => format!("Не удалось открыть службу SysMain: {}", e),
            Self::SysMainStatusQueryFailed(e) => {
                format!("Не удалось получить статус службы SysMain: {}", e)
            }
            Self::SysMainConfigQueryFailed(e) => {
                format!("Не удалось получить конфигурацию службы SysMain: {}", e)
            }
            Self::SysMainEnableFailed(e) => format!("Не удалось включить службу SysMain: {}", e),
            Self::SysMainAlreadyEnabled => "Служба Prefetch уже включена и запущена".to_string(),
            Self::SysMainRequiresAdmin => {
                "Требуются права администратора для включения службы Prefetch".to_string()
            }
            Self::SystemRestoreCheckFailed(e) => {
                format!("Не удалось проверить статус System Restore: {}", e)
            }
            Self::SystemRestoreEnableFailed(e) => {
                format!("Не удалось включить System Restore: {}", e)
            }
            Self::SystemRestoreRequiresAdmin => {
                "Требуются права администратора для включения System Restore".to_string()
            }
            Self::SystemRestoreAlreadyEnabled => {
                "System Restore уже включена на диске C:".to_string()
            }
            Self::WindowsPathNotFound(e) => {
                format!("Не удалось получить путь к Windows: {}", e)
            }
            Self::DirectoryReadFailed(e) => format!("Не удалось прочитать директорию: {}", e),
            Self::RegistryReadFailed(e) => {
                format!("Не удалось прочитать значение реестра: {}", e)
            }
            Self::RegistryWriteFailed(e) => {
                format!("Не удалось записать значение реестра: {}", e)
            }
        }
    }
}
