use egui::CursorIcon::PointingHand as Clickable;

use crate::{custom_widgets::Button, MyApp};

impl MyApp {
    pub fn render_settings_tab(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading("Settings");
                ui.separator();

                if ui
                    .checkbox(
                        &mut self.config.show_only_favorites,
                        "Show only favorite hacks",
                    )
                    .on_hover_cursor(Clickable)
                    .changed()
                {
                    self.config.save_config();
                }

                ui.add_space(10.0);

                if ui
                    .checkbox(
                        &mut self.config.skip_injects_delay,
                        "Skip injects delay (visual)",
                    )
                    .on_hover_cursor(Clickable)
                    .changed()
                {
                    self.config.save_config();
                }

                ui.add_space(10.0);

                if ui
                    .checkbox(&mut self.config.hide_csgo_warning, "Hide CSGO warning")
                    .on_hover_cursor(Clickable)
                    .changed()
                {
                    self.config.save_config();
                }

                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    ui.label("Favorites Color:");
                    if ui
                        .color_edit_button_srgba(&mut self.config.favorites_color)
                        .on_hover_cursor(Clickable)
                        .changed()
                    {
                        self.config.save_config();
                    }
                });

                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    ui.label("API Endpoint:");
                    if ui
                        .text_edit_singleline(&mut self.config.api_endpoint)
                        .changed()
                    {
                        self.config.save_config();
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("CDN Endpoint:");
                    if ui
                        .text_edit_singleline(&mut self.config.cdn_endpoint)
                        .changed()
                    {
                        self.config.save_config();
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("CSGO Injector:");
                    if ui
                        .text_edit_singleline(&mut self.config.csgo_injector)
                        .changed()
                    {
                        self.config.save_config();
                    }
                });

                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    if ui.cbutton("Reset settings").clicked() {
                        self.reset_config();
                        self.toasts.success("Settings reset.");
                    }

                    if ui.cbutton("Open loader folder").clicked() {
                        let downloads_dir = dirs::config_dir()
                            .unwrap_or_else(|| std::path::PathBuf::from("."))
                            .join("anarchyloader");
                        let _ = opener::open(downloads_dir);
                    }
                });
            });
        });
        self.toasts.show(ctx);
    }
}
