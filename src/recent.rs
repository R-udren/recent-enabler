use crate::utils;
use anyhow::{Context, Result};
use std::path::PathBuf;
use winreg::enums::*;

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

pub fn get_recent_info() -> Result<RecentInfo> {
    let recent_path = get_recent_folder()?;
    let stats = utils::get_directory_stats(&recent_path, "lnk")?;

    Ok(RecentInfo {
        lnk_count: stats.count,
        oldest_time: stats.oldest,
        newest_time: stats.newest,
    })
}

pub fn is_recent_disabled() -> Result<bool> {
    let adv_path = r"Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced";
    let exp_path = r"Software\Microsoft\Windows\CurrentVersion\Explorer";

    let track_docs =
        utils::read_reg_dword(HKEY_CURRENT_USER, adv_path, "Start_TrackDocs").unwrap_or(0) == 0;
    let show_recent =
        utils::read_reg_dword(HKEY_CURRENT_USER, exp_path, "ShowRecent").unwrap_or(1) == 0;
    let show_frequent =
        utils::read_reg_dword(HKEY_CURRENT_USER, exp_path, "ShowFrequent").unwrap_or(1) == 0;

    Ok(track_docs || show_recent || show_frequent)
}

pub fn enable_recent() -> Result<()> {
    let adv_path = r"Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced";
    let exp_path = r"Software\Microsoft\Windows\CurrentVersion\Explorer";

    utils::write_reg_dword(HKEY_CURRENT_USER, adv_path, "Start_TrackDocs", 1)?;
    utils::write_reg_dword(HKEY_CURRENT_USER, exp_path, "ShowRecent", 1)?;
    utils::write_reg_dword(HKEY_CURRENT_USER, exp_path, "ShowFrequent", 1)?;

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
