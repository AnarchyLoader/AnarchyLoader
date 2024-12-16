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
                if ui
                    .cselectable_label(self.tab == AppTab::Home, "Home")
                    .clicked()
                {
                    self.tab = AppTab::Home;
                    self.rpc.update(None, Some("Selecting a hack"));
                }
                if ui
                    .cselectable_label(self.tab == AppTab::Settings, "Settings")
                    .clicked()
                {
                    self.tab = AppTab::Settings;
                    self.rpc.update(None, Some("Configuring settings"));
                }
                if ui
                    .cselectable_label(self.tab == AppTab::About, "About")
                    .clicked()
                {
                    self.tab = AppTab::About;
                    self.rpc.update(None, Some("Reading about"));
                }
                if ui
                    .cselectable_label(self.tab == AppTab::Logs, "Logs")
                    .clicked()
                {
                    self.tab = AppTab::Logs;
                    self.rpc.update(None, Some("Viewing Logs"));
                }

                if ctx.input_mut(|i| i.modifiers.shift) || self.tab == AppTab::Debug {
                    if ui
                        .cselectable_label(self.tab == AppTab::Debug, "Debug")
                        .clicked()
                    {
                        self.tab = AppTab::Debug;
                        self.rpc.update(None, Some("🪲 Debugging"));
                    }
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut self.search_query).hint_text("Search..."),
                    );
                });
            });
            ui.add_space(5.0);
        });

        if !self.config.disable_notifications {
            self.toasts.show(ctx);
        }
    }
}
