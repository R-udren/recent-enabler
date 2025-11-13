use anyhow::{Context, Result};
use std::path::PathBuf;
use windows::core::PCWSTR;
use windows::Win32::System::Registry::*;

pub struct RecentInfo {
    pub lnk_count: usize,
    pub oldest_date: Option<String>,
    pub newest_date: Option<String>,
    pub days_since_last: Option<String>,
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

    if !recent_path.exists() {
        return Ok(RecentInfo {
            lnk_count: 0,
            oldest_date: None,
            newest_date: None,
            days_since_last: None,
        });
    }

    let entries: Vec<_> = std::fs::read_dir(&recent_path)
        .context("Не удалось прочитать папку Recent")?
        .filter_map(|e| e.ok())
        .collect();

    let lnk_count = entries
        .iter()
        .filter(|e| {
            e.path()
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.eq_ignore_ascii_case("lnk"))
                .unwrap_or(false)
        })
        .count();

    let mut dates: Vec<std::time::SystemTime> = entries
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
        .collect();

    if dates.is_empty() {
        return Ok(RecentInfo {
            lnk_count,
            oldest_date: None,
            newest_date: None,
            days_since_last: None,
        });
    }

    dates.sort();

    let format_time = |time: std::time::SystemTime| -> String {
        let datetime: chrono::DateTime<chrono::Local> = time.into();
        datetime.format("%d.%m.%Y %H:%M").to_string()
    };

    let oldest_date = dates.first().map(|t| format_time(*t));
    let newest_date = dates.last().map(|t| format_time(*t));

    let days_since_last = if let Some(&newest_time) = dates.last() {
        let now = std::time::SystemTime::now();
        if let Ok(duration) = now.duration_since(newest_time) {
            let days = duration.as_secs() / 86400;
            if days == 0 {
                Some("сегодня".to_string())
            } else if days == 1 {
                Some("1 день назад".to_string())
            } else {
                Some(format!("{} дн. назад", days))
            }
        } else {
            None
        }
    } else {
        None
    };

    Ok(RecentInfo {
        lnk_count,
        oldest_date,
        newest_date,
        days_since_last,
    })
}
pub fn is_recent_disabled() -> Result<bool> {
    unsafe {
        let key_path = "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Advanced";
        let value_name = "Start_TrackDocs";

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
            return Ok(true); // Если ключ не существует, считаем что отключено
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
            Some(std::ptr::addr_of_mut!(data) as *mut u8),
            Some(&mut data_size),
        );

        let _ = RegCloseKey(hkey);

        if result.is_ok() && data_type == REG_DWORD {
            Ok(data == 0) // 0 = отключено, 1 = включено
        } else {
            Ok(true) // Если значение не найдено, считаем что отключено
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

pub fn enable_recent() -> Result<()> {
    unsafe {
        let key_path = "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Advanced";
        let value_name = "Start_TrackDocs";

        let mut hkey = HKEY::default();
        let key_path_wide: Vec<u16> = key_path.encode_utf16().chain(Some(0)).collect();

        let result = RegCreateKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(key_path_wide.as_ptr()),
            Some(0),
            None,
            REG_OPTION_NON_VOLATILE,
            KEY_WRITE,
            None,
            &mut hkey,
            None,
        );

        if result.is_err() {
            return Err(anyhow::anyhow!("Не удалось открыть ключ реестра"));
        }

        let value_name_wide: Vec<u16> = value_name.encode_utf16().chain(Some(0)).collect();
        let data: u32 = 1; // 1 = включено

        let result = RegSetValueExW(
            hkey,
            PCWSTR(value_name_wide.as_ptr()),
            Some(0),
            REG_DWORD,
            Some(std::slice::from_raw_parts(
                std::ptr::addr_of!(data) as *const u8,
                std::mem::size_of::<u32>(),
            )),
        );

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
