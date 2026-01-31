use anyhow::{Context, Result};
use std::path::Path;
use std::time::SystemTime;
use winreg::{RegKey, HKEY};

/// Check if the current process is running with admin privileges
#[must_use]
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

            if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &raw mut token).is_err() {
                return false;
            }

            let mut elevation = TOKEN_ELEVATION::default();
            let mut return_length = 0u32;

            #[allow(clippy::cast_possible_truncation)]
            let result = GetTokenInformation(
                token,
                TokenElevation,
                Some((&raw mut elevation).cast()),
                std::mem::size_of::<TOKEN_ELEVATION>() as u32,
                &raw mut return_length,
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

/// Get statistics about files in a directory
///
/// # Errors
///
/// Returns error if directory cannot be read
pub fn get_directory_stats(path: &Path, extension: &str) -> Result<DirectoryStats> {
    if !path.exists() {
        return Ok(DirectoryStats {
            count: 0,
            oldest: None,
            newest: None,
        });
    }

    let entries = std::fs::read_dir(path)
        .with_context(|| format!("Failed to read directory: {}", path.display()))?;

    let mut count = 0;
    let mut oldest: Option<SystemTime> = None;
    let mut newest: Option<SystemTime> = None;

    for entry in entries.filter_map(std::result::Result::ok) {
        let path = entry.path();
        if path.is_file()
            && path
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case(extension))
        {
            count += 1;
            if let Ok(metadata) = entry.metadata() {
                if let Ok(modified) = metadata.modified() {
                    oldest = Some(oldest.map_or(modified, |t| t.min(modified)));
                    newest = Some(newest.map_or(modified, |t| t.max(modified)));
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

/// Write a DWORD value to the Windows registry
///
/// # Errors
///
/// Returns error if registry key cannot be opened/created or value cannot be written
pub fn write_reg_dword(hkey: HKEY, path: &str, value_name: &str, value: u32) -> Result<()> {
    let (key, _) = RegKey::predef(hkey)
        .create_subkey(path)
        .with_context(|| format!("Failed to open/create registry key: {path}"))?;

    key.set_value(value_name, &value)
        .with_context(|| format!("Failed to set registry value: {value_name}"))
}
