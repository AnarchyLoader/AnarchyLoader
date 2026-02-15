use std::sync::Arc;

use egui::{Context, Id, SystemTheme, Theme::Dark, ThemePreference, ViewportCommand};

use crate::MyApp;

pub(crate) fn register(ctx: &Context) {
    if ctx.data(|d| d.get_temp::<State>(Id::NULL).is_none()) {
        ctx.on_end_pass("update_viewport_theme", Arc::new(State::end_frame));
    }
}

#[derive(Debug, Clone)]
struct State {
    preference: ThemePreference,
}

impl State {
    fn end_frame(ctx: &Context) {
        let preference = ctx.options(|opt| opt.theme_preference);
        let has_changed = !ctx
            .data(|d| d.get_temp::<State>(Id::NULL))
            .map(|s| s.preference)
            .is_some_and(|old| old == preference);
        if has_changed {
            ctx.send_viewport_cmd(ViewportCommand::SetTheme(to_system_theme(preference)));
            ctx.data_mut(|d| d.insert_temp(Id::NULL, State { preference }));
        }
    }
}

impl MyApp {
    pub fn setup_text_animator_color(&mut self, ctx: &Context) {
        // update plain text color used across the UI
        self.ui.text_color = if ctx.theme() == Dark {
            egui::Color32::WHITE
        } else {
            egui::Color32::BLACK
        };
    }
}
fn to_system_theme(preference: ThemePreference) -> SystemTheme {
    match preference {
        ThemePreference::System => SystemTheme::SystemDefault,
        ThemePreference::Dark => SystemTheme::Dark,
        ThemePreference::Light => SystemTheme::Light,
    }
}
