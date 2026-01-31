use crate::{
    error::{RecentEnablerError, Result},
    recent, status, sysmain, system_restore, utils,
};

pub async fn check_recent() -> Result<status::RecentStatus> {
    let path = recent::get_recent_folder()?;
    let is_disabled = recent::is_recent_disabled()?;
    let info = recent::get_recent_info()?;

    Ok(status::RecentStatus {
        path: path.display().to_string(),
        is_disabled,
        files_count: info.lnk_count,
        oldest_time: info.oldest_time,
        newest_time: info.newest_time,
    })
}

pub async fn check_sysmain() -> Result<status::SysMainStatus> {
    let service_status = sysmain::get_sysmain_status()?;
    let startup_type = sysmain::get_sysmain_startup_type()?;
    let prefetch_path = sysmain::get_prefetch_folder()?;

    let (prefetch_count, oldest_time, newest_time, prefetch_error) = match sysmain::get_prefetch_info() {
        Ok(info) => (info.pf_count, info.oldest_time, info.newest_time, None),
        Err(e) => (0, None, None, Some(e.to_string())),
    };

    Ok(status::SysMainStatus {
        is_running: service_status == sysmain::ServiceStatus::Running,
        is_auto: startup_type == sysmain::StartupType::Automatic,
        startup_type: startup_type.as_str().to_string(),
        prefetch_path: prefetch_path.display().to_string(),
        prefetch_count,
        oldest_time,
        newest_time,
        prefetch_error,
    })
}

pub async fn check_system_restore() -> Result<status::SystemRestoreStatus> {
    let is_enabled = system_restore::get_system_restore_info()?;
    Ok(status::SystemRestoreStatus { is_enabled })
}

pub async fn enable_recent() -> Result {
    if !recent::is_recent_disabled()? {
        return Err(RecentEnablerError::RecentAlreadyEnabled);
    }
    recent::enable_recent()?;
    Ok(())
}

pub async fn enable_sysmain() -> Result {
    if !utils::is_admin() {
        return Err(RecentEnablerError::SysMainRequiresAdmin);
    }

    let status = sysmain::get_sysmain_status()?;
    let startup = sysmain::get_sysmain_startup_type()?;

    if status == sysmain::ServiceStatus::Running && startup == sysmain::StartupType::Automatic {
        return Err(RecentEnablerError::SysMainAlreadyEnabled);
    }

    sysmain::enable_sysmain()?;
    Ok(())
}

pub async fn enable_system_restore() -> Result {
    if !utils::is_admin() {
        return Err(RecentEnablerError::SystemRestoreRequiresAdmin);
    }

    let is_enabled = system_restore::get_system_restore_info()?;
    if is_enabled {
        return Err(RecentEnablerError::SystemRestoreAlreadyEnabled);
    }

    system_restore::enable_system_restore()?;
    Ok(())
}
