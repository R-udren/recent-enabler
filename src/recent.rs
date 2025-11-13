use anyhow::{Context, Result};
use std::path::PathBuf;
use windows::core::PCWSTR;
use windows::Win32::System::Registry::*;

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
            oldest_time: None,
            newest_time: None,
        });
    }

    dates.sort();

    Ok(RecentInfo {
        lnk_count,
        oldest_time: dates.first().copied(),
        newest_time: dates.last().copied(),
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

        // 0 = отключено, 1 = включено
        let track_docs_disabled = if result.is_ok() && data_type == REG_DWORD {
            data == 0
        } else {
            true // Если значение не найдено, считаем что отключено
        };

        // Дополнительно учитываем Explorer\ShowRecent и Explorer\ShowFrequent
        let explorer_key_path = "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer";

        let mut show_recent_disabled = true;
        let mut show_frequent_disabled = true;

        // ShowRecent
        let mut hkey_explorer = HKEY::default();
        let explorer_key_path_w: Vec<u16> =
            explorer_key_path.encode_utf16().chain(Some(0)).collect();
        if RegOpenKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(explorer_key_path_w.as_ptr()),
            Some(0),
            KEY_READ,
            &mut hkey_explorer,
        )
        .is_ok()
        {
            let value_name_w: Vec<u16> = "ShowRecent".encode_utf16().chain(Some(0)).collect();
            let mut v: u32 = 0;
            let mut sz = std::mem::size_of::<u32>() as u32;
            let mut ty = REG_NONE;
            if RegQueryValueExW(
                hkey_explorer,
                PCWSTR(value_name_w.as_ptr()),
                None,
                Some(&mut ty),
                Some(std::ptr::addr_of_mut!(v) as *mut u8),
                Some(&mut sz),
            )
            .is_ok()
                && ty == REG_DWORD
            {
                show_recent_disabled = v == 0;
            }

            // ShowFrequent
            let value_name_w: Vec<u16> = "ShowFrequent".encode_utf16().chain(Some(0)).collect();
            let mut v2: u32 = 0;
            let mut sz2 = std::mem::size_of::<u32>() as u32;
            let mut ty2 = REG_NONE;
            if RegQueryValueExW(
                hkey_explorer,
                PCWSTR(value_name_w.as_ptr()),
                None,
                Some(&mut ty2),
                Some(std::ptr::addr_of_mut!(v2) as *mut u8),
                Some(&mut sz2),
            )
            .is_ok()
                && ty2 == REG_DWORD
            {
                show_frequent_disabled = v2 == 0;
            }

            let _ = RegCloseKey(hkey_explorer);
        }

        Ok(track_docs_disabled || show_recent_disabled || show_frequent_disabled)
    }
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

        if result.is_err() {
            return Err(anyhow::anyhow!("Не удалось включить Recent"));
        }

        // Также включаем отображение Recent и Frequent в Проводнике
        let explorer_key_path = "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer";
        let mut hkey_explorer = HKEY::default();
        let explorer_key_path_w: Vec<u16> =
            explorer_key_path.encode_utf16().chain(Some(0)).collect();
        let _ = RegCreateKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(explorer_key_path_w.as_ptr()),
            Some(0),
            None,
            REG_OPTION_NON_VOLATILE,
            KEY_WRITE,
            None,
            &mut hkey_explorer,
            None,
        );

        if !hkey_explorer.is_invalid() {
            let one: u32 = 1;
            for name in ["ShowRecent", "ShowFrequent"] {
                let name_w: Vec<u16> = name.encode_utf16().chain(Some(0)).collect();
                let _ = RegSetValueExW(
                    hkey_explorer,
                    PCWSTR(name_w.as_ptr()),
                    Some(0),
                    REG_DWORD,
                    Some(std::slice::from_raw_parts(
                        std::ptr::addr_of!(one) as *const u8,
                        std::mem::size_of::<u32>(),
                    )),
                );
            }
            let _ = RegCloseKey(hkey_explorer);
        }

        Ok(())
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
