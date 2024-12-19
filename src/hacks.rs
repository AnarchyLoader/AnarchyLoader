use serde::Deserialize;

use crate::utils::downloader::download_file;

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
}

#[derive(Clone, PartialEq, Debug)]
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
}

impl Hack {
    pub(crate) fn new(
        name: &str,
        description: &str,
        author: &str,
        status: &str,
        file: &str,
        process: &str,
        source: &str,
        game: &str,
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
                .unwrap_or_else(|| std::path::PathBuf::from("."))
                .join("anarchyloader")
                .join(&file),
        }
    }

    pub(crate) fn download(&self, file_path: String) -> Result<(), String> {
        if !std::path::Path::new(&file_path).exists() {
            match download_file(&self.file, &file_path) {
                Ok(_) => Ok(()),
                Err(e) => Err(format!("Failed to download file: {}", e)),
            }
        } else {
            Ok(())
        }
    }

    pub(crate) fn fetch_hacks(api_endpoint: &str, lowercase: bool) -> Result<Vec<Hack>, String> {
        match ureq::get(api_endpoint).call() {
            Ok(res) => {
                if res.status() == 200 {
                    let parsed_hacks: Vec<HackApiResponse> =
                        res.into_json().map_err(|e| e.to_string())?;
                    if parsed_hacks.is_empty() {
                        Err("No hacks available.".to_string())
                    } else {
                        log::info!("Fetched {} hacks from API.", parsed_hacks.len());
                        Ok(parsed_hacks
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
                                )
                            })
                            .collect())
                    }
                } else {
                    Err(format!("API request failed with status: {}", res.status()))
                }
            }
            Err(e) => Err(format!("Failed to connect to API: {}", e)),
        }
    }
}

pub(crate) fn get_hack_by_name(hacks: &[Hack], name: &str) -> Option<Hack> {
    hacks.iter().find(|&hack| hack.name == name).cloned()
}

pub(crate) fn get_all_processes(hacks: &[Hack]) -> Vec<String> {
    let mut processes: Vec<String> = hacks.iter().map(|hack| hack.process.clone()).collect();
    processes.sort();
    processes.dedup();
    processes
}
