use crate::{custom_widgets::SelectableLabel, MyApp};

#[derive(Debug, Clone, PartialEq)]
pub enum AppTab {
    Home,
    Settings,
    About,
    Logs,
    Debug,
}

impl Default for AppTab {
    fn default() -> Self {
        AppTab::Home
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
                    AppTab::Home,
                    "Home",
                    "Go to the home screen",
                    &home_rpc_message,
                );
                self.render_tab(
                    ui,
                    AppTab::Settings,
                    "Settings",
                    "Adjust your settings",
                    "Configuring settings",
                );
                self.render_tab(
                    ui,
                    AppTab::About,
                    "About",
                    "Learn more about this loader",
                    "Reading about",
                );
                self.render_tab(ui, AppTab::Logs, "Logs", "Check the logs", "Viewing Logs");

                if ctx.input_mut(|i| i.modifiers.shift) || self.ui.tab == AppTab::Debug {
                    self.render_tab(
                        ui,
                        AppTab::Debug,
                        "Debug",
                        "Get some debug info",
                        "🪲 Debugging",
                    );
                }
            });
            ui.add_space(5.0);
        });

        if !self.app.config.disable_notifications {
            self.toasts.show(ctx);
        }
    }

    fn render_tab(
        &mut self,
        ui: &mut egui::Ui,
        tab: AppTab,
        label: &str,
        tooltip: &str,
        rpc_message: &str,
    ) {
        if ui
            .cselectable_label(self.ui.tab == tab, label)
            .on_hover_text(tooltip)
            .clicked()
        {
            self.ui.tab = tab;
            self.rpc.update(None, Some(rpc_message));
        }
    }
}
