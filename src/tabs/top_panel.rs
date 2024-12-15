use crate::{custom_widgets::SelectableLabel, MyApp};

#[derive(Debug, Clone, PartialEq)]
pub enum AppTab {
    Home,
    Settings,
    About,
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
                    self.rpc.details = "Selecting a hack".to_string();
                    self.rpc.update();
                }
                if ui
                    .cselectable_label(self.tab == AppTab::Settings, "Settings")
                    .clicked()
                {
                    self.tab = AppTab::Settings;
                    self.rpc.details = "Configuring settings".to_string();
                    self.rpc.update();
                }
                if ui
                    .cselectable_label(self.tab == AppTab::About, "About")
                    .clicked()
                {
                    self.tab = AppTab::About;
                    self.rpc.details = "Reading about".to_string();
                    self.rpc.update();
                }
                if ctx.input_mut(|i| i.modifiers.shift) || self.tab == AppTab::Debug {
                    if ui
                        .cselectable_label(self.tab == AppTab::Debug, "Debug")
                        .clicked()
                    {
                        self.tab = AppTab::Debug;
                        self.rpc.details = "🪲 Debugging".to_string();
                        self.rpc.update();
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
    }
}
