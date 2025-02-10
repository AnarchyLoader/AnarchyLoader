use std::{collections::HashSet, fs, path::PathBuf};

use egui::ThemePreference;
use serde::{Deserialize, Serialize};

use crate::{games::local::LocalHack, utils::hacks, MyApp};

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
    pub enable_tab_animations: bool,
    pub transition_duration: f32,
    pub transition_amount: f32,
    pub selected_hack: String,
    pub log_level: log::Level,
    pub skip_update_check: bool,
    pub game_order: Vec<String>,
    pub hidden_games: HashSet<String>,
    pub local_hacks: Vec<LocalHack>,
    pub theme: ThemePreference,
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
            enable_tab_animations: true,
            transition_duration: 0.20,
            transition_amount: 32.0,
            selected_hack: "".to_string(),
            log_level: default_log_level(),
            skip_update_check: false,
            game_order: Vec::new(),
            hidden_games: HashSet::new(),
            local_hacks: Vec::new(),
            theme: ThemePreference::System,
        }
    }
}

impl Config {
    pub fn load() -> Self {
        log::info!("<CONFIG> Loading config");
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("anarchyloader");

        log::debug!("<CONFIG> Config directory: {}", config_dir.display());

        if let Err(e) = fs::create_dir_all(&config_dir) {
            log::warn!("<CONFIG> Failed to create config directory: {}", e);
        }
        let config_path = config_dir.join("config.json");
        log::debug!("<CONFIG> Config path: {}", config_path.display());

        let mut default_config = Config::default();

        if let Ok(data) = fs::read_to_string(&config_path) {
            log::debug!("<CONFIG> Config file found, attempting to read and parse");
            match serde_json::from_str::<Config>(&data) {
                Ok(config) => {
                    log::info!("<CONFIG> Config loaded successfully from file");
                    config
                }
                Err(e) => {
                    log::warn!(
                        "<CONFIG> Failed to parse config file, using default config: {}",
                        e
                    );
                    default_config.update_game_order();
                    default_config
                }
            }
        } else {
            log::info!(
                "<CONFIG> No config file found at {}, creating a new one with default settings",
                config_path.display()
            );
            default_config.update_game_order();
            default_config
        }
    }

    pub fn save(&self) {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("anarchyloader");

        log::debug!("<CONFIG> Config directory: {}", config_dir.display());

        if let Err(e) = fs::create_dir_all(&config_dir) {
            log::warn!("<CONFIG> Failed to create config directory: {}", e);
        }
        let config_path = config_dir.join("config.json");
        log::debug!("<CONFIG> Config path: {}", config_path.display());

        log::debug!("<CONFIG> Serializing config to JSON");
        if let Ok(data) = serde_json::to_string_pretty(&self) {
            log::debug!("<CONFIG> Successfully serialized config to JSON, writing to file");
            if let Err(e) = fs::write(config_path, data) {
                log::error!("<CONFIG> Failed to write config file: {}", e);
            }
        } else {
            log::error!("<CONFIG> Failed to serialize config to JSON");
        }
    }

    pub fn update_game_order(&mut self) {
        log::debug!("<CONFIG> Using API endpoint: {}", self.api_endpoint);
        log::debug!(
            "<CONFIG> Using extra API endpoints: {:?}",
            self.api_extra_endpoints
        );

        let hacks = hacks::fetch_hacks(
            &self.api_endpoint,
            &self.api_extra_endpoints,
            self.lowercase_hacks,
        );

        match hacks {
            Ok(hacks) => {
                log::debug!(
                    "<CONFIG> Successfully fetched {} hacks from API",
                    hacks.len()
                );
                let grouped = MyApp::group_hacks_by_game_internal(&hacks, self);
                self.game_order = grouped.keys().cloned().collect();
                log::info!(
                    "<CONFIG> Game order updated successfully, found {} games",
                    self.game_order.len()
                );
            }
            Err(e) => {
                log::warn!(
                    "<CONFIG> Failed to fetch hacks for updating game order: {}",
                    e
                );
                log::warn!("<CONFIG> Game order update failed, using existing or default order.");
            }
        }
    }

    pub fn reset_game_order(&mut self) {
        log::info!("<CONFIG> Resetting game order");
        log::debug!("<CONFIG> Calling update_game_order to refresh game order from API");
        self.update_game_order();
        log::info!("<CONFIG> Game order reset and updated from API");
        self.save();
        log::debug!("<CONFIG> Config saved after resetting game order");
    }

    pub fn reset(&mut self) {
        log::info!("<CONFIG> Resetting config to default");
        log::debug!("<CONFIG> Setting config to default values");
        *self = Config::default();
        log::debug!("<CONFIG> Config set to default, saving config");
        self.save();
        log::info!("<CONFIG> Config reset to default and saved");
    }
}
