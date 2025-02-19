use egui_material_icons::*;

use crate::{utils::ui::widgets::SelectableLabel, MyApp};

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Default)]
pub enum AppTab {
    #[default]
    Home,
    Settings,
    About,
    Logs,
    Debug,
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
                    AppTab::Home,
                    icons::ICON_HOME,
                    "Home",
                    "Go to the home screen",
                    &home_rpc_message,
                );
                self.render_tab(
                    ui,
                    AppTab::Settings,
                    icons::ICON_SETTINGS,
                    "Settings",
                    "Adjust your settings",
                    "Configuring settings",
                );
                self.render_tab(
                    ui,
                    AppTab::About,
                    icons::ICON_DESCRIPTION,
                    "About",
                    "Learn more about this loader",
                    "Reading about",
                );
                self.render_tab(
                    ui,
                    AppTab::Logs,
                    icons::ICON_EDIT_DOCUMENT,
                    "Logs",
                    "Check the logs",
                    "Viewing Logs",
                );

                if (ctx.input_mut(|i| i.modifiers.shift) && ctx.input_mut(|i| i.modifiers.ctrl))
                    || self.ui.tab == AppTab::Debug
                {
                    self.render_tab(
                        ui,
                        AppTab::Debug,
                        icons::ICON_BUG_REPORT,
                        "Debug",
                        "Get some debug info",
                        "🪲 Debugging",
                    );
                }
            });
            ui.add_space(5.0);
        });

        if !self.app.config.display.disable_toasts {
            self.toasts.show(ctx);
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
        let tab_label = if self.app.config.display.hide_tabs_icons {
            label.to_string()
        } else {
            format!("{} {}", icon, label)
        };

        if ui
            .cselectable_label(self.ui.tab == tab, &tab_label)
            .on_hover_text(tooltip)
            .clicked()
        {
            if self.ui.transitioning {
                return;
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
