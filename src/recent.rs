use anyhow::{Context, Result};
use std::path::PathBuf;
use winreg::enums::*;
use winreg::RegKey;

pub struct RecentInfo {
    pub lnk_count: usize,
    pub oldest_time: Option<std::time::SystemTime>,
    pub newest_time: Option<std::time::SystemTime>,
}

pub fn get_recent_folder() -> Result<PathBuf> {
    let appdata = std::env::var("APPDATA").context("Не удалось получить переменную APPDATA")?;
    Ok(PathBuf::from(appdata)
        .join("Microsoft")
        .join("Windows")
        .join("Recent"))
}

fn count_lnk_files(entries: &[std::fs::DirEntry]) -> usize {
    entries
        .iter()
        .filter(|e| {
            e.path()
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.eq_ignore_ascii_case("lnk"))
                .unwrap_or(false)
        })
        .count()
}

fn collect_file_dates(entries: &[std::fs::DirEntry]) -> Vec<std::time::SystemTime> {
    entries
        .iter()
        .filter(|e| {
            e.path()
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.eq_ignore_ascii_case("lnk"))
                .unwrap_or(false)
        })
        .filter_map(|e| e.metadata().ok())
        .filter_map(|m| m.modified().ok())
        .collect()
}

fn get_oldest_newest(
    mut dates: Vec<std::time::SystemTime>,
) -> (Option<std::time::SystemTime>, Option<std::time::SystemTime>) {
    if dates.is_empty() {
        return (None, None);
    }
    dates.sort();
    (dates.first().copied(), dates.last().copied())
}

pub fn get_recent_info() -> Result<RecentInfo> {
    let recent_path = get_recent_folder()?;

    if !recent_path.exists() {
        return Ok(RecentInfo {
            lnk_count: 0,
            oldest_time: None,
            newest_time: None,
        });
    }

    let entries: Vec<_> = std::fs::read_dir(&recent_path)
        .context("Не удалось прочитать папку Recent")?
        .filter_map(|e| e.ok())
        .collect();

    let lnk_count = count_lnk_files(&entries);
    let dates = collect_file_dates(&entries);
    let (oldest_time, newest_time) = get_oldest_newest(dates);

    Ok(RecentInfo {
        lnk_count,
        oldest_time,
        newest_time,
    })
}

fn read_registry_dword(key: &RegKey, value_name: &str) -> Option<u32> {
    key.get_value::<u32, _>(value_name).ok()
}

fn check_track_docs_disabled() -> Result<bool> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key_path = r"Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced";

    let key = match hkcu.open_subkey(key_path) {
        Ok(k) => k,
        Err(_) => return Ok(true), // If key doesn't exist, assume disabled
    };

    Ok(read_registry_dword(&key, "Start_TrackDocs").unwrap_or(0) == 0)
}

fn check_show_recent_disabled() -> Result<bool> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key_path = r"Software\Microsoft\Windows\CurrentVersion\Explorer";

    let key = match hkcu.open_subkey(key_path) {
        Ok(k) => k,
        Err(_) => return Ok(false), // If key doesn't exist, assume enabled
    };

    Ok(read_registry_dword(&key, "ShowRecent").unwrap_or(1) == 0)
}

fn check_show_frequent_disabled() -> Result<bool> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key_path = r"Software\Microsoft\Windows\CurrentVersion\Explorer";

    let key = match hkcu.open_subkey(key_path) {
        Ok(k) => k,
        Err(_) => return Ok(false), // If key doesn't exist, assume enabled
    };

    Ok(read_registry_dword(&key, "ShowFrequent").unwrap_or(1) == 0)
}

pub fn is_recent_disabled() -> Result<bool> {
    let track_docs = check_track_docs_disabled()?;
    let show_recent = check_show_recent_disabled()?;
    let show_frequent = check_show_frequent_disabled()?;

    Ok(track_docs || show_recent || show_frequent)
}

fn set_registry_dword(key: &RegKey, value_name: &str, value: u32) -> Result<()> {
    key.set_value(value_name, &value)
        .with_context(|| format!("Не удалось записать значение {}", value_name))
}

fn enable_track_docs() -> Result<()> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key_path = r"Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced";

    let (key, _) = hkcu
        .create_subkey(key_path)
        .context("Не удалось открыть ключ реестра Advanced")?;

    set_registry_dword(&key, "Start_TrackDocs", 1)
}

fn enable_show_recent_frequent() -> Result<()> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key_path = r"Software\Microsoft\Windows\CurrentVersion\Explorer";

    let (key, _) = hkcu
        .create_subkey(key_path)
        .context("Не удалось открыть ключ реестра Explorer")?;

    set_registry_dword(&key, "ShowRecent", 1)?;
    set_registry_dword(&key, "ShowFrequent", 1)?;

    Ok(())
}

pub fn enable_recent() -> Result<()> {
    enable_track_docs()?;
    enable_show_recent_frequent()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_recent_folder() {
        let path = get_recent_folder().unwrap();
        assert!(path.to_string_lossy().contains("Recent"));
    }
}
