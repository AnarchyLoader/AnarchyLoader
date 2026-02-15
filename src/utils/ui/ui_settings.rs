use egui::ThemePreference;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Copy)]
pub enum Flavor {
    Latte,
    Frappe,
    Macchiato,
    Mocha,
}

impl Flavor {
    pub fn all() -> [Self; 4] {
        [Self::Latte, Self::Frappe, Self::Macchiato, Self::Mocha]
    }

    pub fn convert(&self) -> catppuccin_egui::Theme {
        match self {
            Flavor::Latte => catppuccin_egui::LATTE,
            Flavor::Frappe => catppuccin_egui::FRAPPE,
            Flavor::Macchiato => catppuccin_egui::MACCHIATO,
            Flavor::Mocha => catppuccin_egui::MOCHA,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DisplaySettings {
    pub favorites_color: egui::Color32,
    pub selected_hack: String,
    pub use_catppuccin_theme: bool,
    pub catpuccin_flavor: Flavor,
    pub disable_hack_name_animation: bool,
    pub hide_steam_account: bool,
    pub hide_statistics: bool,
    pub hide_tabs_icons: bool,
    pub force_use_not_working_hacks: bool,
    pub disable_toasts: bool,
    pub skip_update_check: bool,
    pub show_random_phrase: bool,
    pub theme: ThemePreference,
}

impl Default for DisplaySettings {
    fn default() -> Self {
        DisplaySettings {
            favorites_color: egui::Color32::GOLD,
            selected_hack: "".to_string(),
            use_catppuccin_theme: true,
            catpuccin_flavor: Flavor::Frappe,
            disable_hack_name_animation: false,
            hide_steam_account: false,
            hide_statistics: false,
            hide_tabs_icons: false,
            force_use_not_working_hacks: false,
            disable_toasts: false,
            skip_update_check: false,
            show_random_phrase: true,
            theme: ThemePreference::System,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AnimationSettings {
    pub tab_animations: bool,
    pub duration: f32,
    pub amount: f32,
    pub text_speed: f32,
}

impl Default for AnimationSettings {
    fn default() -> Self {
        AnimationSettings {
            tab_animations: false,
            duration: 0.20,
            amount: 32.0,
            text_speed: 1.5,
        }
    }
}
