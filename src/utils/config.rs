use std::{collections::HashSet, fs, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::{
    games::local::LocalHack,
    utils::{
        api::{api_settings::ApiSettings, hacks},
        ui::ui_settings::{AnimationSettings, DisplaySettings},
    },
    MyApp,
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    pub favorites: HashSet<String>,
    pub show_only_favorites: bool,
    pub automatically_select_hack: bool,
    pub skip_injects_delay: bool,
    pub lowercase_hacks: bool,
    pub disable_rpc: bool,
    pub animations: AnimationSettings,
    pub display: DisplaySettings,
    pub api: ApiSettings,
    pub log_level: log::Level,
    pub game_order: Vec<String>,
    pub hidden_games: HashSet<String>,
    pub local_hacks: Vec<LocalHack>,
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
            automatically_select_hack: true,
            skip_injects_delay: false,
            lowercase_hacks: true,
            disable_rpc: false,
            animations: AnimationSettings::default(),
            display: DisplaySettings::default(),
            api: ApiSettings::default(),
            log_level: default_log_level(),
            game_order: Vec::new(),
            hidden_games: HashSet::new(),
            local_hacks: Vec::new(),
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
        log::debug!("<CONFIG> Using API endpoint: {}", self.api.api_endpoint);
        log::debug!(
            "<CONFIG> Using extra API endpoints: {:?}",
            self.api.api_extra_endpoints
        );

        let hacks = hacks::fetch_hacks(
            &self.api.api_endpoint,
            &self.api.api_extra_endpoints,
            self.lowercase_hacks,
        );

        match hacks {
            Ok(hacks) => {
                log::debug!(
                    "<CONFIG> Successfully fetched {} hacks from API",
                    hacks.len()
                );
                let grouped = MyApp::group_hacks_by_game(&hacks, self);
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
