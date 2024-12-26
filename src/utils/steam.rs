use std::{fs, path::PathBuf};

use vdf_reader::{entry::Table, Reader};
use winreg::{
    enums::{HKEY_LOCAL_MACHINE, KEY_READ},
    RegKey,
};

#[derive(Debug)]
pub struct SteamAccount {
    pub username: String,
    pub name: String,
}

impl SteamAccount {
    fn locate_steam() -> Result<PathBuf, String> {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let installation_regkey = hklm
            .open_subkey_with_flags("SOFTWARE\\Wow6432Node\\Valve\\Steam", KEY_READ)
            .or_else(|_| hklm.open_subkey_with_flags("SOFTWARE\\Valve\\Steam", KEY_READ))
            .map_err(|e| format!("Failed to open Steam registry key: {}", e))?;

        let install_path_str: String = installation_regkey
            .get_value("InstallPath")
            .map_err(|e| format!("Failed to get InstallPath: {}", e))?;

        Ok(PathBuf::from(install_path_str))
    }

    fn parse_user() -> Result<Self, String> {
        let path = Self::locate_steam()?.join("config").join("loginusers.vdf");

        let raw = fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read loginusers.vdf: {}", e))?;

        let mut reader = Reader::from(raw.as_str());
        let file = Table::load(&mut reader).map_err(|e| format!("Failed to parse VDF: {}", e))?;

        let users = file
            .get("users")
            .ok_or("Missing users table")?
            .as_table()
            .ok_or("Invalid users table")?;

        users
            .iter()
            .find_map(|(_, user_data)| {
                let user_info = user_data.as_table()?;
                if user_info.get("MostRecent").and_then(|v| v.as_str()) == Some("1") {
                    Some((
                        user_info
                            .get("AccountName")
                            .and_then(|v| v.as_str())?
                            .to_string(),
                        user_info
                            .get("PersonaName")
                            .and_then(|v| v.as_str())?
                            .to_string(),
                    ))
                } else {
                    None
                }
            })
            .map(|(username, name)| Self { username, name })
            .ok_or("No recent user found".to_string())
    }

    pub fn new() -> Result<Self, String> {
        Self::parse_user()
    }

    pub fn default() -> Self {
        Self {
            username: "unknown".to_string(),
            name: "unknown".to_string(),
        }
    }
}
