//! Recent Files service - business logic only.

use crate::domain::{
    CheckSeverity, FileStats, OperationResult, RecentInfo, RecentStatus, RegistryCheck, Result,
};
use crate::repositories::{file_system, registry};
use std::path::PathBuf;

const TRACK_DOCS_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced";
const EXPLORER_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Explorer";
const POLICY_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Policies\Explorer";

fn get_recent_path() -> Result<PathBuf> {
    let appdata = std::env::var("APPDATA")?;
    Ok(PathBuf::from(appdata).join("Microsoft\\Windows\\Recent"))
}

fn build_checks() -> Vec<RegistryCheck> {
    vec![
        RegistryCheck {
            name: "Track Documents".into(),
            key: TRACK_DOCS_KEY.into(),
            value: "Start_TrackDocs".into(),
            expected: 1,
            actual: None,
            severity: CheckSeverity::Important,
            is_policy: false,
        },
        RegistryCheck {
            name: "Track Programs".into(),
            key: TRACK_DOCS_KEY.into(),
            value: "Start_TrackProgs".into(),
            expected: 1,
            actual: None,
            severity: CheckSeverity::Important,
            is_policy: false,
        },
        RegistryCheck {
            name: "Show Recent".into(),
            key: EXPLORER_KEY.into(),
            value: "ShowRecent".into(),
            expected: 1,
            actual: None,
            severity: CheckSeverity::Minor,
            is_policy: false,
        },
        RegistryCheck {
            name: "Show Frequent".into(),
            key: EXPLORER_KEY.into(),
            value: "ShowFrequent".into(),
            expected: 1,
            actual: None,
            severity: CheckSeverity::Minor,
            is_policy: false,
        },
        RegistryCheck {
            name: "Policy Block".into(),
            key: POLICY_KEY.into(),
            value: "NoRecentDocsHistory".into(),
            expected: 0,
            actual: None,
            severity: CheckSeverity::Critical,
            is_policy: true,
        },
        RegistryCheck {
            name: "Menu Policy".into(),
            key: POLICY_KEY.into(),
            value: "NoRecentDocsMenu".into(),
            expected: 0,
            actual: None,
            severity: CheckSeverity::Critical,
            is_policy: true,
        },
    ]
}

fn check_status(checks: &[RegistryCheck]) -> RecentStatus {
    let policy_blocked = checks.iter().any(|c| c.is_policy && !c.is_ok());

    if policy_blocked {
        return RecentStatus::PolicyBlocked;
    }

    let important_failed = checks
        .iter()
        .filter(|c| matches!(c.severity, CheckSeverity::Important))
        .all(|c| !c.is_ok());

    if important_failed {
        return RecentStatus::FullyDisabled;
    }

    let any_failed = checks.iter().any(|c| !c.is_ok());
    if any_failed {
        RecentStatus::PartiallyEnabled
    } else {
        RecentStatus::FullyEnabled
    }
}

pub fn get_info() -> Result<RecentInfo> {
    let path = get_recent_path()?;
    let files = file_system::scan_folder(&path, "lnk").unwrap_or_else(|_| FileStats::empty());

    let mut checks = build_checks();
    for check in &mut checks {
        // Check HKCU first
        let mut val = registry::read_hkcu(&check.key, &check.value);

        // For policies, also check HKLM if HKCU is OK or missing
        if check.is_policy && (val.is_none() || val == Some(check.expected)) {
            if let Some(hklm_val) = registry::read_hklm(&check.key, &check.value) {
                val = Some(hklm_val);
            }
        }

        check.actual = val;
    }

    let status = check_status(&checks);

    Ok(RecentInfo {
        path: path.display().to_string(),
        files,
        status,
        checks,
    })
}

pub fn enable() -> Result<OperationResult> {
    let checks = build_checks();
    let mut fixed = 0;
    let mut needs_admin = false;

    for check in checks {
        if check.is_policy {
            let mut policy_fixed = false;
            // Try to fix in HKCU
            if registry::write_hkcu(&check.key, &check.value, check.expected).is_ok() {
                policy_fixed = true;
            }

            // Also try to fix in HKLM (requires admin)
            if registry::write_hklm(&check.key, &check.value, check.expected).is_err() {
                needs_admin = true;
            } else {
                policy_fixed = true;
            }

            if policy_fixed {
                fixed += 1;
            }
            continue;
        }

        if registry::write_hkcu(&check.key, &check.value, check.expected).is_ok() {
            fixed += 1;
        }
    }

    let mut res = OperationResult::success(format!("Enabled {} settings", fixed));
    if needs_admin {
        res.message
            .push_str(" (some policy settings require admin)");
        res = res.requires_admin();
    }
    Ok(res)
}
