use egui::ThemePreference;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DisplaySettings {
    pub favorites_color: egui::Color32,
    pub selected_hack: String,
    pub disable_hack_name_animation: bool,
    pub hide_steam_account: bool,
    pub hide_statistics: bool,
    pub hide_tabs_icons: bool,
    pub force_unworking_hacks: bool,
    pub disable_toasts: bool,
    pub skip_update_check: bool,
    pub theme: ThemePreference,
}

impl Default for DisplaySettings {
    fn default() -> Self {
        DisplaySettings {
            favorites_color: egui::Color32::GOLD,
            selected_hack: "".to_string(),
            disable_hack_name_animation: false,
            hide_steam_account: false,
            hide_statistics: false,
            hide_tabs_icons: false,
            force_unworking_hacks: false,
            disable_toasts: false,
            skip_update_check: false,
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
            tab_animations: true,
            duration: 0.20,
            amount: 32.0,
            text_speed: 1.5,
        }
    }
}
