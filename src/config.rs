use std::collections::HashSet;

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
}

fn default_favorites_color() -> egui::Color32 {
    egui::Color32::GOLD
}

fn default_api_endpoint() -> String {
    "https://anarchy.collapseloader.org/api/hacks/".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Config {
            favorites: HashSet::new(),
            show_only_favorites: false,
            favorites_color: default_favorites_color(),
            skip_injects_delay: false,
            api_endpoint: default_api_endpoint(),
        }
    }
}
