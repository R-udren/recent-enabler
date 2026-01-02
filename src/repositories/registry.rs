//! Registry access helpers - thin wrapper over winreg.

use crate::domain::{AppError, Result};
use winreg::{enums::*, RegKey, HKEY};

pub fn read_dword(hive: HKEY, key: &str, value: &str) -> Option<u32> {
    RegKey::predef(hive)
        .open_subkey(key)
        .ok()?
        .get_value::<u32, _>(value)
        .ok()
}

pub fn write_dword(hive: HKEY, key: &str, value: &str, data: u32) -> Result<()> {
    let root = RegKey::predef(hive);
    let (key, _) = root
        .create_subkey(key)
        .map_err(|e| AppError::Registry(e.to_string()))?;
    key.set_value(value, &data)
        .map_err(|e| AppError::Registry(e.to_string()))
}

pub fn read_hkcu(key: &str, value: &str) -> Option<u32> {
    read_dword(HKEY_CURRENT_USER, key, value)
}

pub fn write_hkcu(key: &str, value: &str, data: u32) -> Result<()> {
    write_dword(HKEY_CURRENT_USER, key, value, data)
}

pub fn write_hklm(key: &str, value: &str, data: u32) -> Result<()> {
    write_dword(HKEY_LOCAL_MACHINE, key, value, data)
}

pub fn read_hklm(key: &str, value: &str) -> Option<u32> {
    read_dword(HKEY_LOCAL_MACHINE, key, value)
}
