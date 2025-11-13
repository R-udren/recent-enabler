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
pub fn get_recent_file_dates() -> Result<(Option<String>, Option<String>)> {
    let recent_path = get_recent_folder()?;

    if !recent_path.exists() {
        return Ok((None, None));
    }

    let mut dates: Vec<std::time::SystemTime> = std::fs::read_dir(&recent_path)
        .context("Не удалось прочитать папку Recent")?
        .filter_map(|e| e.ok())
        .filter_map(|e| e.metadata().ok())
        .filter_map(|m| m.modified().ok())
        .collect();

    if dates.is_empty() {
        return Ok((None, None));
    }

    dates.sort();

    let format_time = |time: std::time::SystemTime| -> String {
        let datetime: chrono::DateTime<chrono::Local> = time.into();
        datetime.format("%d.%m.%Y %H:%M").to_string()
    };

    let oldest = dates.first().map(|t| format_time(*t));
    let newest = dates.last().map(|t| format_time(*t));

    Ok((oldest, newest))
}

pub fn get_days_since_last_recent() -> Result<Option<String>> {
    let recent_path = get_recent_folder()?;

    if !recent_path.exists() {
        return Ok(None);
    }

    let newest_time = std::fs::read_dir(&recent_path)
        .context("Не удалось прочитать папку Recent")?
        .filter_map(|e| e.ok())
        .filter_map(|e| e.metadata().ok())
        .filter_map(|m| m.modified().ok())
        .max();

    if let Some(time) = newest_time {
        let now = std::time::SystemTime::now();
        if let Ok(duration) = now.duration_since(time) {
            let days = duration.as_secs() / 86400;
            if days == 0 {
                Ok(Some("сегодня".to_string()))
            } else if days == 1 {
                Ok(Some("1 день назад".to_string()))
            } else {
                Ok(Some(format!("{} дн. назад", days)))
            }
        } else {
            Ok(None)
        }
    } else {
        Ok(None)
    }
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
            Err(anyhow::anyhow!("Не удалось включить Recent"))
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
