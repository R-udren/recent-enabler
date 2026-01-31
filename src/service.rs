use crate::{recent, status, sysmain, system_restore, utils};

pub async fn check_recent() -> std::result::Result<status::RecentStatus, String> {
    let path = recent::get_recent_folder().map_err(|e| e.to_string())?;
    let is_disabled = recent::is_recent_disabled().map_err(|e| e.to_string())?;
    let info = recent::get_recent_info().map_err(|e| e.to_string())?;

    Ok(status::RecentStatus {
        path: path.display().to_string(),
        is_disabled,
        files_count: info.lnk_count,
        oldest_time: info.oldest_time,
        newest_time: info.newest_time,
    })
}

pub async fn check_sysmain() -> std::result::Result<status::SysMainStatus, String> {
    let service_status = sysmain::get_sysmain_status().map_err(|e| e.to_string())?;
    let startup_type = sysmain::get_sysmain_startup_type().map_err(|e| e.to_string())?;
    let prefetch_path = sysmain::get_prefetch_folder().map_err(|e| e.to_string())?;

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

pub async fn check_system_restore() -> std::result::Result<status::SystemRestoreStatus, String> {
    let is_enabled = system_restore::get_system_restore_info().map_err(|e| e.to_string())?;
    Ok(status::SystemRestoreStatus { is_enabled })
}

pub async fn enable_recent() -> std::result::Result<String, String> {
    if !recent::is_recent_disabled().map_err(|e| e.to_string())? {
        return Ok("Запись в Recent уже включена!".to_string());
    }
    recent::enable_recent().map_err(|e| e.to_string())?;
    Ok("Запись в Recent успешно включена!".to_string())
}

pub async fn enable_sysmain() -> std::result::Result<String, String> {
    if !utils::is_admin() {
        return Err("Требуются права администратора для включения службы Prefetch!".to_string());
    }

    let status = sysmain::get_sysmain_status().map_err(|e| e.to_string())?;
    let startup = sysmain::get_sysmain_startup_type().map_err(|e| e.to_string())?;

    if status == sysmain::ServiceStatus::Running && startup == sysmain::StartupType::Automatic {
        return Ok("Служба Prefetch уже включена и запущена!".to_string());
    }

    sysmain::enable_sysmain().map_err(|e| e.to_string())?;
    Ok("Служба Prefetch успешно включена и запущена!".to_string())
}

pub async fn enable_system_restore() -> std::result::Result<String, String> {
    if !utils::is_admin() {
        return Err("Требуются права администратора для включения System Restore!".to_string());
    }

    system_restore::enable_system_restore().map_err(|e| e.to_string())?;
    Ok("System Restore успешно включена на диске C:!".to_string())
}
