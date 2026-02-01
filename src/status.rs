use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Helper to convert SystemTime to Unix timestamp
fn system_time_to_timestamp(time: &SystemTime) -> u64 {
    time.duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Custom serializer for Option<SystemTime> -> Option<u64>
fn serialize_system_time<S>(time: &Option<SystemTime>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match time {
        Some(t) => serializer.serialize_some(&system_time_to_timestamp(t)),
        None => serializer.serialize_none(),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentStatus {
    pub path: String,
    pub is_disabled: bool,
    pub files_count: usize,

    #[serde(serialize_with = "serialize_system_time")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oldest_time: Option<SystemTime>,

    #[serde(serialize_with = "serialize_system_time")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub newest_time: Option<SystemTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SysMainStatus {
    pub is_running: bool,
    pub is_auto: bool,
    pub startup_type: String,
    pub prefetch_path: String,
    pub prefetch_count: usize,

    #[serde(serialize_with = "serialize_system_time")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oldest_time: Option<SystemTime>,

    #[serde(serialize_with = "serialize_system_time")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub newest_time: Option<SystemTime>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefetch_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemRestoreStatus {
    pub is_enabled: bool,
}
