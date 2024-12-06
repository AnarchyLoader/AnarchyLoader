use std::{collections::HashSet, fs, path::PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    #[serde(default)]
    pub favorites: HashSet<String>,
    #[serde(default)]
    pub show_only_favorites: bool,
    #[serde(default = "default_favorites_color")]
    pub favorites_color: egui::Color32,
    #[serde(default)]
    pub skip_injects_delay: bool,
    #[serde(default = "default_api_endpoint")]
    pub api_endpoint: String,
    #[serde(default = "default_cdn_endpoint")]
    pub cdn_endpoint: String,
}

fn default_favorites_color() -> egui::Color32 {
    egui::Color32::GOLD
}

fn default_api_endpoint() -> String {
    "https://api.anarchy.my/api/hacks/".to_string()
}

fn default_cdn_endpoint() -> String {
    "https://cdn.collapseloader.org/anarchy/".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Config {
            favorites: HashSet::new(),
            show_only_favorites: false,
            favorites_color: default_favorites_color(),
            skip_injects_delay: false,
            api_endpoint: default_api_endpoint(),
            cdn_endpoint: default_cdn_endpoint(),
        }
    }
}

impl Config {
    pub fn load_config() -> Self {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("anarchyloader");

        fs::create_dir_all(&config_dir).ok();
        let config_path = config_dir.join("config.json");

        if let Ok(data) = fs::read_to_string(&config_path) {
            serde_json::from_str::<Config>(&data).unwrap_or_default()
        } else {
            Config::default()
        }
    }

    pub fn save_config(&self) {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("anarchyloader");

        fs::create_dir_all(&config_dir).ok();
        let config_path = config_dir.join("config.json");

        if let Ok(data) = serde_json::to_string(&self) {
            fs::write(config_path, data).ok();
        }
    }
}
