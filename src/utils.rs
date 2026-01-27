use anyhow::{Context, Result};
use std::path::Path;
use std::time::SystemTime;
use winreg::{RegKey, HKEY};

pub fn is_admin() -> bool {
    #[cfg(windows)]
    {
        use windows::Win32::Foundation::{CloseHandle, HANDLE};
        use windows::Win32::Security::{
            GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY,
        };
        use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

        unsafe {
            let mut token = HANDLE::default();

            if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token).is_err() {
                return false;
            }

            let mut elevation = TOKEN_ELEVATION::default();
            let mut return_length = 0u32;

            let result = GetTokenInformation(
                token,
                TokenElevation,
                Some(&mut elevation as *mut _ as *mut _),
                std::mem::size_of::<TOKEN_ELEVATION>() as u32,
                &mut return_length,
            );

            let _ = CloseHandle(token);

            result.is_ok() && elevation.TokenIsElevated != 0
        }
    }

    #[cfg(not(windows))]
    {
        false
    }
}

pub struct DirectoryStats {
    pub count: usize,
    pub oldest: Option<SystemTime>,
    pub newest: Option<SystemTime>,
}

pub fn get_directory_stats(path: &Path, extension: &str) -> Result<DirectoryStats> {
    if !path.exists() {
        return Ok(DirectoryStats {
            count: 0,
            oldest: None,
            newest: None,
        });
    }

    let entries =
        std::fs::read_dir(path).with_context(|| format!("Failed to read directory: {:?}", path))?;

    let mut count = 0;
    let mut oldest: Option<SystemTime> = None;
    let mut newest: Option<SystemTime> = None;

    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file()
            && path
                .extension()
                .and_then(|s| s.to_str())
                .map(|s| s.eq_ignore_ascii_case(extension))
                .unwrap_or(false)
        {
            count += 1;
            if let Ok(metadata) = entry.metadata() {
                if let Ok(modified) = metadata.modified() {
                    oldest = Some(oldest.map(|t| t.min(modified)).unwrap_or(modified));
                    newest = Some(newest.map(|t| t.max(modified)).unwrap_or(modified));
                }
            }
        }
    }

    Ok(DirectoryStats {
        count,
        oldest,
        newest,
    })
}

pub fn read_reg_dword(hkey: HKEY, path: &str, value: &str) -> Option<u32> {
    RegKey::predef(hkey)
        .open_subkey(path)
        .ok()
        .and_then(|k| k.get_value(value).ok())
}

pub fn write_reg_dword(hkey: HKEY, path: &str, value_name: &str, value: u32) -> Result<()> {
    let (key, _) = RegKey::predef(hkey)
        .create_subkey(path)
        .with_context(|| format!("Failed to open/create registry key: {}", path))?;

    key.set_value(value_name, &value)
        .with_context(|| format!("Failed to set registry value: {}", value_name))
}
