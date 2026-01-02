//! Elevation helpers - relaunch the current executable as admin.

use crate::domain::{AppError, Result};
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use windows::core::PCWSTR;
use windows::Win32::UI::Shell::ShellExecuteW;
use windows::Win32::UI::WindowsAndMessaging::SW_SHOW;

pub fn run_as_admin() -> Result<()> {
    // Get current exe path
    let exe = std::env::current_exe().map_err(|e| AppError::Other(e.to_string()))?;

    // Build null-terminated wide strings
    let exe_wide: Vec<u16> = exe
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    let op_wide: Vec<u16> = OsStr::new("runas")
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    unsafe {
        let hinst = ShellExecuteW(
            None,
            PCWSTR(op_wide.as_ptr()),
            PCWSTR(exe_wide.as_ptr()),
            PCWSTR::null(),
            PCWSTR::null(),
            SW_SHOW,
        );

        // Per docs, return value > 32 indicates success
        let rv = hinst.0 as isize;
        if rv <= 32 {
            return Err(AppError::Other(format!(
                "ShellExecuteW failed: code {}",
                rv
            )));
        }
    }

    Ok(())
}
