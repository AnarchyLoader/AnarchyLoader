use std::{collections::HashSet, fs, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::{games::local::LocalHack, hacks, MyApp};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    pub favorites: HashSet<String>,
    pub show_only_favorites: bool,
    pub favorites_color: egui::Color32,
    pub automatically_select_hack: bool,
    pub skip_injects_delay: bool,
    pub lowercase_hacks: bool,
    pub api_endpoint: String,
    pub api_extra_endpoints: Vec<String>,
    pub cdn_endpoint: String,
    pub cdn_extra_endpoints: Vec<String>,
    pub hide_steam_account: bool,
    pub hide_statistics: bool,
    pub hide_tabs_icons: bool,
    pub disable_notifications: bool,
    pub disable_rpc: bool,
    pub selected_hack: String,
    pub log_level: log::Level,
    pub skip_update_check: bool,
    pub game_order: Vec<String>,
    pub local_hacks: Vec<LocalHack>,
}

fn default_favorites_color() -> egui::Color32 {
    egui::Color32::GOLD
}

pub(crate) fn default_api_endpoint() -> String {
    "https://api.anarchy.my/api/hacks/".to_string()
}

pub(crate) fn default_api_extra_endpoints() -> Vec<String> {
    vec!["https://anarchy.ttfdk.lol/api/hacks/".to_string()]
}

pub(crate) fn default_cdn_endpoint() -> String {
    "https://cdn.anarchy.my/".to_string()
}

pub(crate) fn default_cdn_extra_endpoint() -> Vec<String> {
    vec!["https://axkanxneklh7.objectstorage.eu-amsterdam-1.oci.customer-oci.com/n/axkanxneklh7/b/anarchy/o/".to_string()]
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
            api_extra_endpoints: default_api_extra_endpoints(),
            cdn_endpoint: default_cdn_endpoint(),
            cdn_extra_endpoints: default_cdn_extra_endpoint(),
            hide_steam_account: false,
            hide_statistics: false,
            hide_tabs_icons: false,
            disable_notifications: false,
            disable_rpc: false,
            selected_hack: "".to_string(),
            log_level: default_log_level(),
            skip_update_check: false,
            game_order: Vec::new(),
            local_hacks: Vec::new()
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

        let mut default_config = Config::default();

        if let Ok(data) = fs::read_to_string(&config_path) {
            serde_json::from_str::<Config>(&data).unwrap_or_else(|_| {
                default_config.update_game_order();
                default_config
            })
        } else {
            log::info!("No config file found, creating a new one");
            default_config.update_game_order();
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

    pub fn update_game_order(&mut self) {
        log::info!("Updating game order");
        let hacks = match hacks::fetch_hacks(
            &self.api_endpoint,
            &self.api_extra_endpoints,
            self.lowercase_hacks,
        ) {
            Ok(h) => h,
            Err(_) => Vec::new(),
        };
        let grouped = MyApp::group_hacks_by_game_internal(&hacks, self);
        self.game_order = grouped.keys().cloned().collect();
    }

    pub fn reset_game_order(&mut self) {
        log::info!("Resetting game order");
        let hacks = match hacks::fetch_hacks(
            &self.api_endpoint,
            &self.api_extra_endpoints,
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
