use std::{collections::HashSet, fs, path::PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    #[serde(default)]
    pub favorites: HashSet<String>,
    #[serde(default)]
    pub show_only_favorites: bool,
    #[serde(default = "default_favorites_color")]
    pub favorites_color: egui::Color32,
    #[serde(default)]
    pub automatically_select_hack: bool,
    #[serde(default)]
    pub skip_injects_delay: bool,
    #[serde(default)]
    pub lowercase_hacks: bool,
    #[serde(default = "default_api_endpoint")]
    pub api_endpoint: String,
    #[serde(default = "default_cdn_endpoint")]
    pub cdn_endpoint: String,
    #[serde(default = "default_csgo_injector")]
    pub csgo_injector: String,
    #[serde(default)]
    pub hide_csgo_warning: bool,
    #[serde(default)]
    pub hide_steam_account: bool,
    #[serde(default)]
    pub disable_notifications: bool,
    #[serde(default = "selected_hack")]
    pub selected_hack: String,
}

fn default_favorites_color() -> egui::Color32 {
    egui::Color32::GOLD
}

pub(crate) fn default_api_endpoint() -> String {
    "https://api.anarchy.my/api/hacks/".to_string()
}

pub(crate) fn default_cdn_endpoint() -> String {
    "https://cdn.collapseloader.org/anarchy/".to_string()
}

pub(crate) fn default_csgo_injector() -> String {
    "AnarchyInjector.exe".to_string()
}

fn selected_hack() -> String {
    "".to_string()
}

impl Default for Config {
    fn default() -> Self {
        // default config
        Config {
            favorites: HashSet::new(),
            show_only_favorites: false,
            favorites_color: default_favorites_color(),
            automatically_select_hack: false,
            skip_injects_delay: false,
            lowercase_hacks: false,
            api_endpoint: default_api_endpoint(),
            cdn_endpoint: default_cdn_endpoint(),
            csgo_injector: default_csgo_injector(),
            hide_csgo_warning: false,
            hide_steam_account: false,
            disable_notifications: false,
            selected_hack: selected_hack(),
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
