use anyhow::{Context, Result};
use std::path::PathBuf;
use windows::core::PCWSTR;
use windows::Win32::System::Registry::*;
pub fn get_recent_folder() -> Result<PathBuf> {
    let appdata = std::env::var("APPDATA").context("Не удалось получить переменную APPDATA")?;
    Ok(PathBuf::from(appdata)
        .join("Microsoft")
        .join("Windows")
        .join("Recent"))
}
pub fn is_recent_disabled() -> Result<bool> {
    unsafe {
        let key_path = "Software\\Microsoft\\Windows\\CurrentVersion\\Policies\\Explorer";
        let value_name = "NoRecentDocsHistory";

        let mut hkey = HKEY::default();
        let key_path_wide: Vec<u16> = key_path.encode_utf16().chain(Some(0)).collect();

        let result = RegOpenKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(key_path_wide.as_ptr()),
            Some(0),
            KEY_READ,
            &mut hkey,
        );

        if result.is_err() {
            return Ok(false);
        }

        let value_name_wide: Vec<u16> = value_name.encode_utf16().chain(Some(0)).collect();
        let mut data: u32 = 0;
        let mut data_size = std::mem::size_of::<u32>() as u32;
        let mut data_type = REG_NONE;

        let result = RegQueryValueExW(
            hkey,
            PCWSTR(value_name_wide.as_ptr()),
            None,
            Some(&mut data_type),
            Some(&mut data as *mut u32 as *mut u8),
            Some(&mut data_size),
        );

        let _ = RegCloseKey(hkey);

        if result.is_ok() && data_type == REG_DWORD {
            Ok(data == 1)
        } else {
            Ok(false)
        }
    }
}
pub fn is_recent_folder_empty() -> Result<bool> {
    let recent_path = get_recent_folder()?;

    if !recent_path.exists() {
        return Ok(true);
    }

    let entries = std::fs::read_dir(&recent_path)
        .context("Не удалось прочитать папку Recent")?
        .filter_map(|e| e.ok())
        .count();

    Ok(entries == 0)
}
pub fn get_recent_files_count() -> Result<usize> {
    let recent_path = get_recent_folder()?;

    if !recent_path.exists() {
        return Ok(0);
    }

    let count = std::fs::read_dir(&recent_path)
        .context("Не удалось прочитать папку Recent")?
        .filter_map(|e| e.ok())
        .count();

    Ok(count)
}
pub fn get_recent_folder_size() -> Result<u64> {
    let recent_path = get_recent_folder()?;

    if !recent_path.exists() {
        return Ok(0);
    }

    let total_size = std::fs::read_dir(&recent_path)
        .context("Не удалось прочитать папку Recent")?
        .filter_map(|e| e.ok())
        .filter_map(|e| e.metadata().ok())
        .map(|m| m.len())
        .sum();

    Ok(total_size)
}
pub fn enable_recent() -> Result<()> {
    unsafe {
        let key_path = "Software\\Microsoft\\Windows\\CurrentVersion\\Policies\\Explorer";
        let value_name = "NoRecentDocsHistory";

        let mut hkey = HKEY::default();
        let key_path_wide: Vec<u16> = key_path.encode_utf16().chain(Some(0)).collect();

        let result = RegOpenKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(key_path_wide.as_ptr()),
            Some(0),
            KEY_WRITE,
            &mut hkey,
        );

        if result.is_err() {
            return Ok(());
        }

        let value_name_wide: Vec<u16> = value_name.encode_utf16().chain(Some(0)).collect();
        let result = RegDeleteValueW(hkey, PCWSTR(value_name_wide.as_ptr()));

        let _ = RegCloseKey(hkey);

        if result.is_ok() {
            Ok(())
        } else {
            Ok(())
        }
    }
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
