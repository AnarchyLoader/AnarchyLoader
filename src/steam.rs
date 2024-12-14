use std::{fs, path::PathBuf};

use vdf_reader::{entry::Table, Reader};

#[derive(Debug)]
pub struct SteamAccount {
    pub username: String,
    pub name: String,
}

impl SteamAccount {
    pub fn new() -> Result<Self, String> {
        let path = PathBuf::from("C:\\Program Files (x86)\\Steam\\config\\loginusers.vdf");
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

    pub fn default() -> Self {
        Self {
            username: "unknown".to_string(),
            name: "unknown".to_string(),
        }
    }
}
