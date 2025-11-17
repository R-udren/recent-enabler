use std::process::Command;
use winreg::enums::*;
use winreg::RegKey;

fn is_system_restore_enabled() -> std::io::Result<bool> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    if let Ok(system_restore) =
        hklm.open_subkey("SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\SystemRestore")
    {
        let rpsession_interval: Result<u32, _> = system_restore.get_value("RPSessionInterval");
        return Ok(rpsession_interval.unwrap_or(0) == 1);
    }
    Ok(false) // Cannot access key assume disabled
}

fn enable_system_restore(drive: &str) -> std::io::Result<()> {
    let output = Command::new("powershell")
        .args([
            "-Command",
            &format!("Enable-ComputerRestore -Drive '{}'", drive),
        ])
        .output()?;

    if output.status.success() {
        println!("System Restore enabled on drive {}", drive);
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let essential = stderr
            .lines()
            .find(|line| !line.trim().is_empty())
            .unwrap_or(stderr.as_ref());
        eprintln!("Failed to enable System Restore: {}", essential);
    }
    Ok(())
}

fn main() -> std::io::Result<()> {
    let enabled = is_system_restore_enabled()?;
    println!("System Restore enabled: {}", enabled);

    if !enabled {
        println!("Enabling System Restore...");
        enable_system_restore("C:")?;
    }

    Ok(())
}
