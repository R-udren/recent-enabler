//! Recent Files functionality detection and management.
//!
//! This module provides comprehensive detection of all registry settings
//! that affect Recent Files functionality, with severity levels and
//! Group Policy detection.

#![allow(dead_code)]

use crate::domain::{
    CheckSeverity, DomainResult, OperationResult, RecentCheck, RecentCheckResult, RecentInfo,
    RecentStatus, RegistryLocation,
};
use anyhow::{Context, Result};
use std::cmp::{max, min};
use std::path::PathBuf;
use std::time::SystemTime;
use winreg::enums::*;
use winreg::{RegKey, HKEY};

// =============================================================================
// Registry Check Definitions
// =============================================================================

/// Returns all registry checks that affect Recent Files functionality.
/// These are ordered by severity (Critical -> Important -> Minor).
fn get_all_recent_checks() -> Vec<RecentCheck> {
    vec![
        // === CRITICAL CHECKS (Policy - may not be overridable) ===
        RecentCheck {
            source: "Group Policy: Не сохранять историю недавних документов".into(),
            location: RegistryLocation::HKCU,
            key_path: r"Software\Microsoft\Windows\CurrentVersion\Policies\Explorer".into(),
            value_name: "NoRecentDocsHistory".into(),
            current_value: None,
            expected_for_enabled: 0,
            severity: CheckSeverity::Critical,
            is_policy: true,
        },
        RecentCheck {
            source: "Group Policy (Machine): Не сохранять историю недавних документов".into(),
            location: RegistryLocation::HKLM,
            key_path: r"Software\Microsoft\Windows\CurrentVersion\Policies\Explorer".into(),
            value_name: "NoRecentDocsHistory".into(),
            current_value: None,
            expected_for_enabled: 0,
            severity: CheckSeverity::Critical,
            is_policy: true,
        },
        RecentCheck {
            source: "Group Policy: Очищать историю недавних документов при выходе".into(),
            location: RegistryLocation::HKCU,
            key_path: r"Software\Microsoft\Windows\CurrentVersion\Policies\Explorer".into(),
            value_name: "ClearRecentDocsOnExit".into(),
            current_value: None,
            expected_for_enabled: 0,
            severity: CheckSeverity::Critical,
            is_policy: true,
        },
        // === IMPORTANT CHECKS (User settings - main functionality) ===
        RecentCheck {
            source: "Отслеживание открытых документов".into(),
            location: RegistryLocation::HKCU,
            key_path: r"Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced".into(),
            value_name: "Start_TrackDocs".into(),
            current_value: None,
            expected_for_enabled: 1,
            severity: CheckSeverity::Important,
            is_policy: false,
        },
        RecentCheck {
            source: "Отслеживание запуска программ".into(),
            location: RegistryLocation::HKCU,
            key_path: r"Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced".into(),
            value_name: "Start_TrackProgs".into(),
            current_value: None,
            expected_for_enabled: 1,
            severity: CheckSeverity::Important,
            is_policy: false,
        },
        // === MINOR CHECKS (UI display only) ===
        RecentCheck {
            source: "Показывать недавние элементы в меню Пуск".into(),
            location: RegistryLocation::HKCU,
            key_path: r"Software\Microsoft\Windows\CurrentVersion\Explorer".into(),
            value_name: "ShowRecent".into(),
            current_value: None,
            expected_for_enabled: 1,
            severity: CheckSeverity::Minor,
            is_policy: false,
        },
        RecentCheck {
            source: "Показывать часто используемые элементы".into(),
            location: RegistryLocation::HKCU,
            key_path: r"Software\Microsoft\Windows\CurrentVersion\Explorer".into(),
            value_name: "ShowFrequent".into(),
            current_value: None,
            expected_for_enabled: 1,
            severity: CheckSeverity::Minor,
            is_policy: false,
        },
        RecentCheck {
            source: "Недавние файлы в диалогах открытия (NoFileMru)".into(),
            location: RegistryLocation::HKCU,
            key_path: r"Software\Microsoft\Windows\CurrentVersion\Policies\Comdlg32".into(),
            value_name: "NoFileMru".into(),
            current_value: None,
            expected_for_enabled: 0,
            severity: CheckSeverity::Minor,
            is_policy: false,
        },
        RecentCheck {
            source: "Недавние папки в диалогах открытия (NoPlacesBar)".into(),
            location: RegistryLocation::HKCU,
            key_path: r"Software\Microsoft\Windows\CurrentVersion\Policies\Comdlg32".into(),
            value_name: "NoPlacesBar".into(),
            current_value: None,
            expected_for_enabled: 0,
            severity: CheckSeverity::Minor,
            is_policy: false,
        },
    ]
}

// =============================================================================
// Registry Operations
// =============================================================================

/// Read a DWORD value from registry, returning None if not found.
fn read_registry_dword_from_hive(hive: HKEY, key_path: &str, value_name: &str) -> Option<u32> {
    let root = RegKey::predef(hive);
    root.open_subkey(key_path)
        .ok()
        .and_then(|key| key.get_value::<u32, _>(value_name).ok())
}

/// Read a registry check and populate its current_value.
fn read_registry_check(mut check: RecentCheck) -> RecentCheck {
    let value = match check.location {
        RegistryLocation::HKCU => {
            read_registry_dword_from_hive(HKEY_CURRENT_USER, &check.key_path, &check.value_name)
        }
        RegistryLocation::HKLM => {
            read_registry_dword_from_hive(HKEY_LOCAL_MACHINE, &check.key_path, &check.value_name)
        }
        RegistryLocation::Both => {
            // Check HKLM first (policy), then HKCU
            read_registry_dword_from_hive(HKEY_LOCAL_MACHINE, &check.key_path, &check.value_name)
                .or_else(|| {
                    read_registry_dword_from_hive(
                        HKEY_CURRENT_USER,
                        &check.key_path,
                        &check.value_name,
                    )
                })
        }
    };
    check.current_value = value;
    check
}

/// Write a DWORD value to registry.
fn write_registry_dword(hive: HKEY, key_path: &str, value_name: &str, value: u32) -> Result<()> {
    let root = RegKey::predef(hive);
    let (key, _) = root
        .create_subkey(key_path)
        .with_context(|| format!("Failed to create/open key: {}", key_path))?;
    key.set_value(value_name, &value)
        .with_context(|| format!("Failed to write value: {}", value_name))?;
    Ok(())
}

// =============================================================================
// Path Operations
// =============================================================================

/// Get the path to the Recent folder.
pub fn get_recent_folder() -> Result<PathBuf> {
    let appdata = std::env::var("APPDATA").context("Failed to get APPDATA environment variable")?;
    Ok(PathBuf::from(appdata)
        .join("Microsoft")
        .join("Windows")
        .join("Recent"))
}

// =============================================================================
// File Scanning (Single Pass)
// =============================================================================

/// Scan Recent folder and get file statistics in a single pass.
fn scan_recent_folder(path: &PathBuf) -> Result<(usize, Option<SystemTime>, Option<SystemTime>)> {
    if !path.exists() {
        return Ok((0, None, None));
    }

    let entries = std::fs::read_dir(path).context("Failed to read Recent folder")?;

    // Single pass - no intermediate collections
    let (count, oldest, newest) = entries
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext.eq_ignore_ascii_case("lnk"))
                .unwrap_or(false)
        })
        .filter_map(|e| e.metadata().ok().and_then(|m| m.modified().ok()))
        .fold(
            (0usize, None::<SystemTime>, None::<SystemTime>),
            |(count, oldest, newest), time| {
                (
                    count + 1,
                    Some(oldest.map_or(time, |old| min(old, time))),
                    Some(newest.map_or(time, |new| max(new, time))),
                )
            },
        );

    Ok((count, oldest, newest))
}

// =============================================================================
// Status Detection
// =============================================================================

/// Get comprehensive Recent Files status by checking all registry sources.
pub fn get_recent_status() -> DomainResult<RecentStatus> {
    let checks: Vec<RecentCheck> = get_all_recent_checks()
        .into_iter()
        .map(read_registry_check)
        .collect();

    // Check for policy blocks (cannot override)
    let policy_blocks: Vec<&RecentCheck> = checks
        .iter()
        .filter(|c| c.is_policy && c.is_blocking())
        .collect();

    if !policy_blocks.is_empty() {
        let policy_sources = policy_blocks.iter().map(|c| c.source.clone()).collect();
        return Ok(RecentStatus::DisabledByPolicy { policy_sources });
    }

    // Count failures by severity
    let critical_checks: Vec<&RecentCheck> = checks
        .iter()
        .filter(|c| c.severity == CheckSeverity::Critical)
        .collect();
    let important_checks: Vec<&RecentCheck> = checks
        .iter()
        .filter(|c| c.severity == CheckSeverity::Important)
        .collect();

    let critical_failed = critical_checks.iter().filter(|c| c.is_blocking()).count();
    let important_failed = important_checks.iter().filter(|c| c.is_blocking()).count();
    let important_total = important_checks.len();

    // Determine overall status
    if critical_failed > 0 || important_failed == important_total {
        Ok(RecentStatus::FullyDisabled)
    } else if important_failed > 0 || checks.iter().any(|c| c.is_blocking()) {
        let enabled_features: Vec<String> = checks
            .iter()
            .filter(|c| c.is_enabled())
            .map(|c| c.source.clone())
            .collect();
        let disabled_features: Vec<String> = checks
            .iter()
            .filter(|c| c.is_blocking())
            .map(|c| c.source.clone())
            .collect();
        Ok(RecentStatus::PartiallyEnabled {
            enabled_features,
            disabled_features,
        })
    } else {
        Ok(RecentStatus::FullyEnabled)
    }
}

/// Get all check results with fixability information.
pub fn get_recent_check_results() -> DomainResult<Vec<RecentCheckResult>> {
    let checks: Vec<RecentCheck> = get_all_recent_checks()
        .into_iter()
        .map(read_registry_check)
        .collect();

    let results = checks
        .into_iter()
        .map(|check| {
            let can_fix = !check.is_policy || check.location == RegistryLocation::HKCU;
            let fix_description = if check.is_blocking() {
                if can_fix {
                    Some(format!(
                        "Установить {} = {}",
                        check.value_name, check.expected_for_enabled
                    ))
                } else {
                    Some("Требуется изменение групповой политики".to_string())
                }
            } else {
                None
            };
            RecentCheckResult {
                check,
                can_fix,
                fix_description,
            }
        })
        .collect();

    Ok(results)
}

/// Get complete Recent Files information.
pub fn get_recent_info() -> DomainResult<RecentInfo> {
    let path = get_recent_folder()?;
    let (lnk_count, oldest_time, newest_time) = scan_recent_folder(&path)?;
    let status = get_recent_status()?;
    let checks = get_recent_check_results()?;

    Ok(RecentInfo {
        path: path.display().to_string(),
        lnk_count,
        oldest_time,
        newest_time,
        status,
        checks,
    })
}

// =============================================================================
// Enable Operations
// =============================================================================

/// Enable all non-policy Recent Files registry settings.
pub fn enable_recent() -> DomainResult<OperationResult> {
    let checks = get_all_recent_checks();
    let mut fixed_count = 0;
    let mut failed_count = 0;
    let mut policy_blocked = 0;

    for check in checks {
        // Skip policy checks that are in HKLM (require admin/GPO)
        if check.is_policy && check.location == RegistryLocation::HKLM {
            let current = read_registry_check(check.clone());
            if current.is_blocking() {
                policy_blocked += 1;
            }
            continue;
        }

        // Get hive for this check
        let hive = match check.location {
            RegistryLocation::HKCU => HKEY_CURRENT_USER,
            RegistryLocation::HKLM => HKEY_LOCAL_MACHINE,
            RegistryLocation::Both => HKEY_CURRENT_USER, // Prefer HKCU for user settings
        };

        // Try to set the value
        match write_registry_dword(
            hive,
            &check.key_path,
            &check.value_name,
            check.expected_for_enabled,
        ) {
            Ok(()) => fixed_count += 1,
            Err(_) => failed_count += 1,
        }
    }

    if policy_blocked > 0 {
        Ok(OperationResult::failure(format!(
            "Включено {} настроек, {} заблокировано политикой, {} ошибок",
            fixed_count, policy_blocked, failed_count
        )))
    } else if failed_count > 0 {
        Ok(OperationResult::failure(format!(
            "Включено {} настроек, {} ошибок",
            fixed_count, failed_count
        )))
    } else {
        Ok(OperationResult::success(format!(
            "Успешно включено {} настроек Recent Files",
            fixed_count
        )))
    }
}

/// Enable only specific checks by source name.
#[allow(dead_code)]
pub fn enable_recent_checks(sources: &[&str]) -> DomainResult<OperationResult> {
    let checks = get_all_recent_checks();
    let mut fixed_count = 0;
    let mut errors = Vec::new();

    for check in checks {
        if !sources.iter().any(|s| check.source.contains(s)) {
            continue;
        }

        if check.is_policy && check.location == RegistryLocation::HKLM {
            errors.push(format!("{}: заблокировано политикой", check.source));
            continue;
        }

        let hive = match check.location {
            RegistryLocation::HKCU => HKEY_CURRENT_USER,
            RegistryLocation::HKLM => HKEY_LOCAL_MACHINE,
            RegistryLocation::Both => HKEY_CURRENT_USER,
        };

        match write_registry_dword(
            hive,
            &check.key_path,
            &check.value_name,
            check.expected_for_enabled,
        ) {
            Ok(()) => fixed_count += 1,
            Err(e) => errors.push(format!("{}: {}", check.source, e)),
        }
    }

    if errors.is_empty() {
        Ok(OperationResult::success(format!(
            "Включено {} настроек",
            fixed_count
        )))
    } else {
        Ok(OperationResult::failure(format!(
            "Включено {}, ошибки: {}",
            fixed_count,
            errors.join("; ")
        )))
    }
}

// =============================================================================
// Legacy API Compatibility
// =============================================================================

/// Legacy function: Check if Recent Files is disabled (any critical/important check fails).
pub fn is_recent_disabled() -> Result<bool> {
    let status = get_recent_status()?;
    Ok(!status.is_enabled())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_recent_folder() {
        let path = get_recent_folder().unwrap();
        assert!(path.to_string_lossy().contains("Recent"));
    }

    #[test]
    fn test_get_all_checks() {
        let checks = get_all_recent_checks();
        assert!(!checks.is_empty());
        assert!(checks.iter().any(|c| c.severity == CheckSeverity::Critical));
        assert!(checks
            .iter()
            .any(|c| c.severity == CheckSeverity::Important));
        assert!(checks.iter().any(|c| c.severity == CheckSeverity::Minor));
    }
}
