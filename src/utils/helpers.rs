use sysinfo::System;
use winreg::{enums::HKEY_LOCAL_MACHINE, RegKey};

pub fn get_windows_version() -> Option<String> {
    let hkey = RegKey::predef(HKEY_LOCAL_MACHINE);

    let key = hkey
        .open_subkey("SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion")
        .ok()?;

    let product_name: String = key.get_value("ProductName").ok()?;
    let version: String = key.get_value("DisplayVersion").ok()?;
    let release_id: String = key.get_value("ReleaseId").ok()?;
    let build: String = key.get_value("CurrentBuild").ok()?;
    Some(format!(
        "{} (Version: {}, Release ID: {}, Build: {})",
        product_name, version, release_id, build
    ))
}

pub fn start_cs_prompt() -> Result<(), String> {
    opener::open("steam://launch/730/dialog")
        .map_err(|e| format!("Failed to open Counter-Strike: {}", e))
}

pub fn is_process_running(process_name: &str) -> bool {
    let mut system = System::new_all();
    system.refresh_all();
    for process in system.processes_by_name(process_name.as_ref()) {
        if process.name() == process_name {
            return true;
        }
    }
    false
}
