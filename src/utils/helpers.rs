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
