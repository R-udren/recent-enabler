use std::time::SystemTime;

#[derive(Debug, Clone)]
pub struct RecentStatus {
    pub path: String,
    pub is_disabled: bool,
    pub files_count: usize,
    pub oldest_time: Option<SystemTime>,
    pub newest_time: Option<SystemTime>,
}

#[derive(Debug, Clone)]
pub struct SysMainStatus {
    pub is_running: bool,
    pub is_auto: bool,
    pub startup_type: String,
    pub prefetch_path: String,
    pub prefetch_count: usize,
    pub oldest_time: Option<SystemTime>,
    pub newest_time: Option<SystemTime>,
    pub prefetch_error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SystemRestoreStatus {
    pub is_enabled: bool,
}
