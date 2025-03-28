use serde::{Deserialize, Serialize};

use crate::utils::api::downloader::download_file;

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub(crate) struct HackApiResponse {
    pub name: String,
    pub description: String,
    pub author: String,
    pub status: String,
    pub file: String,
    pub process: String,
    pub source: String,
    pub game: String,
    pub working: bool,
    pub steam_module: bool,
    pub id: i32,
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub(crate) struct Hack {
    pub name: String,
    pub description: String,
    pub author: String,
    pub status: String,
    pub file: String,
    pub process: String,
    pub source: String,
    pub game: String,
    pub file_path: std::path::PathBuf,
    pub local: bool,
    pub arch: String,
    pub working: bool,
    pub steam_module: bool,
    pub id: i32,
}

impl Hack {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        name: &str,
        description: &str,
        author: &str,
        status: &str,
        file: &str,
        process: &str,
        source: &str,
        game: &str,
        local: bool,
        working: bool,
        steam_module: bool,
        id: i32,
    ) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            author: author.to_string(),
            status: status.to_string(),
            file: file.to_string(),
            process: process.to_string(),
            source: source.to_string(),
            game: game.to_string(),
            file_path: dirs::config_dir()
                .unwrap_or_else(|| std::path::PathBuf::from("../../../../.."))
                .join("anarchyloader")
                .join(file),
            local,
            arch: String::new(),
            working,
            steam_module,
            id,
        }
    }

    pub(crate) fn download(&self, file_path: String) -> Result<(), String> {
        if !std::path::Path::new(&file_path).exists() {
            match download_file(&self.file, None) {
                Ok(_) => Ok(()),
                Err(e) => Err(format!("{}", e)),
            }
        } else {
            Ok(())
        }
    }

    pub(crate) fn download_steam_module(&self) -> Result<(), String> {
        match download_file(&format!("steam_{}", self.file), None) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("{}", e)),
        }
    }
}

impl Default for Hack {
    fn default() -> Self {
        Self {
            name: "n/a".to_string(),
            description: "n/a".to_string(),
            author: "???".to_string(),
            status: "n/a".to_string(),
            file: "n/a".to_string(),
            process: "n/a".to_string(),
            source: "n/a".to_string(),
            game: "n/a".to_string(),
            file_path: std::path::PathBuf::new(),
            local: false,
            arch: "n/a".to_string(),
            working: true,
            steam_module: false,
            id: 0,
        }
    }
}

pub(crate) fn fetch_hacks(
    api_endpoint: &str,
    api_extra_endpoints: &[String],
    lowercase: bool,
) -> Result<(Vec<Hack>, bool), String> {
    let mut endpoints = vec![api_endpoint.to_string()];
    endpoints.extend(api_extra_endpoints.to_owned());

    for endpoint in endpoints {
        match ureq::get(&format!("{}hacks/", &endpoint)).call() {
            Ok(res) => {
                if res.status() == 200 {
                    let parsed_hacks: Vec<HackApiResponse> =
                        res.into_json().map_err(|e| e.to_string())?;
                    return if parsed_hacks.is_empty() {
                        Err("No hacks available.".to_string())
                    } else {
                        log::info!(
                            "<HACKS> Successfully fetched {} hacks from API",
                            parsed_hacks.len()
                        );
                        let hacks: Vec<Hack> = parsed_hacks
                            .into_iter()
                            .map(|hack| {
                                let name = if lowercase {
                                    hack.name.to_lowercase()
                                } else {
                                    hack.name.clone()
                                };
                                let description = if lowercase {
                                    hack.description.to_lowercase()
                                } else {
                                    hack.description.clone()
                                };
                                Hack::new(
                                    &name,
                                    &description,
                                    &hack.author,
                                    &hack.status,
                                    &hack.file,
                                    &hack.process,
                                    &hack.source,
                                    &hack.game,
                                    false,
                                    hack.working,
                                    hack.steam_module,
                                    hack.id,
                                )
                            })
                            .collect();
                        return Ok((hacks, false));
                    };
                }
            }
            Err(e) => log::warn!("Failed to connect to {}: {}", endpoint, e),
        }
    }

    match load_cached_hacks() {
        Ok(cached_hacks) => {
            log::info!("<HACKS> Loaded hacks from cache.");
            Ok((cached_hacks, true))
        }
        Err(e) => Err(format!(
            "All endpoints failed and no cache available: {}",
            e
        )),
    }
}

pub(crate) fn get_hack_by_name(hacks: &[Hack], name: &str) -> Option<Hack> {
    hacks.iter().find(|&hack| hack.name == name).cloned()
}

pub(crate) fn get_hack_by_dll(hacks: &[Hack], dll: &str) -> Option<Hack> {
    hacks.iter().find(|&hack| hack.file == dll).cloned()
}

fn load_cached_hacks() -> Result<Vec<Hack>, String> {
    let cache_path = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("../../../../.."))
        .join("anarchyloader")
        .join("hacks_cache.json");

    if cache_path.exists() {
        let data = std::fs::read_to_string(&cache_path).map_err(|e| e.to_string())?;
        let hacks: Vec<Hack> = serde_json::from_str(&data).map_err(|e| e.to_string())?;
        Ok(hacks)
    } else {
        Err("Cache file does not exist.".to_string())
    }
}

pub(crate) fn save_hacks_to_cache(hacks: &[Hack]) -> Result<(), String> {
    let cache_path = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("../../../../.."))
        .join("anarchyloader")
        .join("hacks_cache.json");

    let data = serde_json::to_string(hacks).map_err(|e| e.to_string())?;
    std::fs::write(cache_path, data).map_err(|e| e.to_string())
}
