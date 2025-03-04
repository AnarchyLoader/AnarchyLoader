use std::{fmt, sync::LazyLock};

use egui_material_icons::{icons::ICON_FAVORITE, *};
use rand::prelude::IndexedRandom;

use crate::{tabs::top_panel::AppTab::*, utils::ui::widgets::SelectableLabel, MyApp};

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Default, Hash)]
pub enum AppTab {
    #[default]
    Home,
    Settings,
    About,
    Logs,
    Debug,
}

impl AppTab {
    pub fn icon(&self) -> &'static str {
        match self {
            Home => icons::ICON_HOME,
            Settings => icons::ICON_SETTINGS,
            About => icons::ICON_DESCRIPTION,
            Logs => icons::ICON_EDIT_DOCUMENT,
            Debug => icons::ICON_BUG_REPORT,
        }
    }
}

impl fmt::Display for AppTab {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Home => "Home",
            Settings => "Settings",
            About => "About",
            Logs => "Logs",
            Debug => "Debug",
        };
        write!(f, "{}", s)
    }
}

static RANDOM_PHRASES: LazyLock<Vec<String>> = LazyLock::new(|| {
    vec![
        "by dest4590".to_string(),
        "thanks for using.".to_string(),
        format!("made with {}", ICON_FAVORITE),
        "did you know that loader using own injector?".to_string(),
        "loader is open source!".to_string(),
        "loader is written in rust!".to_string(),
        "contributions are welcome!".to_string(),
        "stay tuned for updates!".to_string(),
    ]
});

#[derive(Debug)]
pub struct TopPanel {
    pub(crate) random_phrase: String,
}

impl Default for TopPanel {
    fn default() -> Self {
        Self {
            random_phrase: RANDOM_PHRASES.choose(&mut rand::rng()).unwrap().to_string(),
        }
    }
}

impl MyApp {
    pub fn render_top_panel(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.add_space(5.0);
            ui.horizontal(|ui| {
                let home_rpc_message = if let Some(ref hack) = self.app.selected_hack {
                    format!("Selected {}", hack.name)
                } else {
                    "Selecting hack".to_string()
                };

                self.render_tab(
                    ui,
                    Home,
                    Home.icon(),
                    "Home",
                    "Go to the home screen",
                    &home_rpc_message,
                );
                self.render_tab(
                    ui,
                    Settings,
                    Settings.icon(),
                    "Settings",
                    "Adjust your settings",
                    "Configuring settings",
                );
                self.render_tab(
                    ui,
                    About,
                    About.icon(),
                    "About",
                    "Learn more about this loader",
                    "Reading about",
                );
                self.render_tab(
                    ui,
                    Logs,
                    Logs.icon(),
                    "Logs",
                    "Check the logs",
                    "Viewing Logs",
                );

                if (ctx.input_mut(|i| i.modifiers.shift) && ctx.input_mut(|i| i.modifiers.ctrl))
                    || self.ui.tab == Debug
                {
                    self.render_tab(
                        ui,
                        Debug,
                        Debug.icon(),
                        "Debug",
                        "Get some debug info",
                        "🪲 Debugging",
                    );
                }

                if self.app.config.display.show_random_phrase {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(self.ui.tabs.top_panel.random_phrase.clone());
                    });
                }
            });
            ui.add_space(5.0);
        });

        if !self.app.config.display.disable_toasts {
            self.toasts.show(ctx);
        }
    }

    pub fn tab_label(&mut self, tab: AppTab, icon: &str, label: &str) -> String {
        if self.app.config.display.hide_tabs_icons {
            tab.to_string()
        } else {
            format!("{} {}", icon, label)
        }
    }

    fn render_tab(
        &mut self,
        ui: &mut egui::Ui,
        tab: AppTab,
        icon: &str,
        label: &str,
        tooltip: &str,
        rpc_message: &str,
    ) {
        if ui
            .cselectable_label(
                self.ui.tab == tab,
                &self.tab_label(tab.clone(), icon, label),
            )
            .on_hover_text(tooltip)
            .clicked()
        {
            if self.ui.transitioning {
                return;
            }

            if self.ui.tab != tab.clone() {
                self.ui.tabs.top_panel.random_phrase =
                    RANDOM_PHRASES.choose(&mut rand::rng()).unwrap().to_string();
            }

            self.ui.tab = tab.clone();

            if !self.app.config.disable_rpc {
                self.rpc.update(
                    None,
                    Some(rpc_message),
                    Some(&format!("{:?}", tab).to_lowercase()),
                );
            }
        }
    }
}
