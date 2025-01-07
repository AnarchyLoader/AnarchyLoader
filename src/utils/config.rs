use std::{collections::HashSet, fs, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::{hacks, MyApp};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    pub favorites: HashSet<String>,
    pub show_only_favorites: bool,
    pub favorites_color: egui::Color32,
    pub automatically_select_hack: bool,
    pub skip_injects_delay: bool,
    pub lowercase_hacks: bool,
    pub api_endpoint: String,
    pub api_endpoint_fallback: String,
    pub cdn_endpoint: String,
    pub cdn_fallback_endpoint: String,
    pub hide_steam_account: bool,
    pub hide_statistics: bool,
    pub disable_notifications: bool,
    pub disable_rpc: bool,
    pub selected_hack: String,
    pub log_level: log::Level,
    pub game_order: Vec<String>,
}

fn default_favorites_color() -> egui::Color32 {
    egui::Color32::GOLD
}

pub(crate) fn default_api_endpoint() -> String {
    "https://api.anarchy.my/api/hacks/".to_string()
}

pub(crate) fn default_api_fallback_endpoint() -> String {
    "https://anarchy.ttfdk.lol/api/hacks/".to_string()
}

pub(crate) fn default_cdn_endpoint() -> String {
    "https://cdn.collapseloader.org/anarchy/".to_string()
}

pub(crate) fn default_cdn_fallback_endpoint() -> String {
    "https://cdn-ru.collapseloader.org/anarchy/".to_string()
}

pub(crate) fn default_log_level() -> log::Level {
    log::Level::Info
}

impl Default for Config {
    fn default() -> Self {
        // default config
        Config {
            favorites: HashSet::new(),
            show_only_favorites: false,
            favorites_color: default_favorites_color(),
            automatically_select_hack: true,
            skip_injects_delay: false,
            lowercase_hacks: true,
            api_endpoint: default_api_endpoint(),
            api_endpoint_fallback: default_api_fallback_endpoint(),
            cdn_endpoint: default_cdn_endpoint(),
            cdn_fallback_endpoint: default_cdn_fallback_endpoint(),
            hide_steam_account: false,
            hide_statistics: false,
            disable_notifications: false,
            disable_rpc: false,
            selected_hack: "".to_string(),
            log_level: default_log_level(),
            game_order: Vec::new(),
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("anarchyloader");

        fs::create_dir_all(&config_dir).ok();
        let config_path = config_dir.join("config.json");

        if let Ok(data) = fs::read_to_string(&config_path) {
            serde_json::from_str::<Config>(&data).unwrap_or_else(|_| {
                let mut default_config = Config::default();

                let hacks = match hacks::fetch_hacks(
                    &default_config.api_endpoint,
                    &default_config.api_endpoint_fallback,
                    default_config.lowercase_hacks,
                ) {
                    Ok(hacks) => hacks,
                    Err(_) => Vec::new(),
                };
                let grouped_hacks = MyApp::group_hacks_by_game_internal(&hacks, &default_config);
                default_config.game_order = grouped_hacks.keys().cloned().collect();
                default_config
            })
        } else {
            let mut default_config = Config::default();

            let hacks = match hacks::fetch_hacks(
                &default_config.api_endpoint,
                &default_config.api_endpoint_fallback,
                default_config.lowercase_hacks,
            ) {
                Ok(hacks) => hacks,
                Err(_) => Vec::new(),
            };
            let grouped_hacks = MyApp::group_hacks_by_game_internal(&hacks, &default_config);
            default_config.game_order = grouped_hacks.keys().cloned().collect();
            default_config
        }
    }

    pub fn save(&self) {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("anarchyloader");

        fs::create_dir_all(&config_dir).ok();
        let config_path = config_dir.join("config.json");

        if let Ok(data) = serde_json::to_string(&self) {
            fs::write(config_path, data).ok();
        }
    }

    pub fn reset_game_order(&mut self) {
        let hacks = match hacks::fetch_hacks(
            &self.api_endpoint,
            &self.api_endpoint_fallback,
            self.lowercase_hacks,
        ) {
            Ok(h) => h,
            Err(_) => Vec::new(),
        };
        let grouped = MyApp::group_hacks_by_game_internal(&hacks, self);
        self.game_order = grouped.keys().cloned().collect();
        self.save();
    }

    pub fn reset(&mut self) {
        *self = Config::default();
        self.save();
    }
}
