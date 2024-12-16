use egui::{CursorIcon::PointingHand as Clickable, RichText};
use egui_modal::Modal;

use crate::{
    config::{default_api_endpoint, default_cdn_endpoint, default_cdn_fallback_endpoint},
    custom_widgets::{Button, CheckBox, TextEdit},
    hacks, MyApp,
};

impl MyApp {
    pub fn render_settings_tab(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .drag_to_scroll(false)
                .show(ui, |ui| {
                    ui.heading("Settings");
                    ui.separator();

                    // MARK: - Display Options
                    ui.group(|ui| {
                        ui.label("Display Options:");
                        ui.add_space(5.0);

                        if ui
                            .ccheckbox(
                                &mut self.config.show_only_favorites,
                                "Show only favorite hacks",
                            )
                            .changed()
                        {
                            self.config.save_config();
                        }
                        if ui
                            .ccheckbox(&mut self.config.lowercase_hacks, "Lowercase hack names")
                            .changed()
                        {
                            self.hacks = match hacks::Hack::fetch_hacks(
                                &self.config.api_endpoint,
                                self.config.lowercase_hacks,
                            ) {
                                Ok(hacks) => hacks,
                                Err(_err) => {
                                    self.main_menu_message = "Failed to fetch hacks.".to_string();
                                    Vec::new()
                                }
                            };

                            self.toasts.info("Hacks refreshed.");
                            self.config.save_config();
                        };
                        if ui
                            .ccheckbox(&mut self.config.hide_steam_account, "Hide Steam account")
                            .changed()
                        {
                            self.config.save_config();
                        }
                    });

                    ui.add_space(10.0);

                    // MARK: - Injection/Delay Options
                    ui.group(|ui| {
                        ui.label("Injection/Delay Options:");
                        ui.add_space(5.0);

                        if ui
                            .ccheckbox(
                                &mut self.config.skip_injects_delay,
                                "Skip injects delay (visual)",
                            )
                            .changed()
                        {
                            self.config.save_config();
                        }
                        if ui
                            .ccheckbox(
                                &mut self.config.automatically_select_hack,
                                "Automatically select recently injected hack",
                            )
                            .changed()
                        {
                            self.config.save_config();
                        }
                    });

                    ui.add_space(10.0);

                    // MARK: - Notifications/Warnings
                    ui.group(|ui| {
                        ui.label("Notifications/Warnings:");
                        ui.add_space(5.0);
                        if ui
                            .ccheckbox(&mut self.config.hide_csgo_warning, "Hide CSGO/CS2 warning")
                            .changed()
                        {
                            self.config.save_config();
                        }
                        if ui
                            .ccheckbox(
                                &mut self.config.disable_notifications,
                                "Disable notifications",
                            )
                            .changed()
                        {
                            self.config.save_config();
                        }
                    });

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

                    ui.label("Right-click the input field to reset these text settings.");

                    ui.add_space(2.0);

                    ui.horizontal(|ui| {
                        ui.label("API Endpoint:");
                        if ui
                            .ctext_edit(&mut self.config.api_endpoint, default_api_endpoint())
                            .changed()
                        {
                            self.config.save_config();
                        }
                    });

                    ui.add_space(2.0);

                    ui.horizontal(|ui| {
                        ui.label("CDN Endpoint:");
                        if ui
                            .ctext_edit(&mut self.config.cdn_endpoint, default_cdn_endpoint())
                            .changed()
                        {
                            self.config.save_config();
                        }
                    });

                    ui.add_space(2.0);

                    ui.horizontal(|ui| {
                        ui.label("CDN Fallback Endpoint:");
                        if ui
                            .ctext_edit(
                                &mut self.config.cdn_fallback_endpoint,
                                default_cdn_fallback_endpoint(),
                            )
                            .changed()
                        {
                            self.config.save_config();
                        }
                    });

                    ui.add_space(10.0);

                    ui.horizontal(|ui| {
                        if ui.button("Open loader folder").clicked() {
                            let downloads_dir = dirs::config_dir()
                                .unwrap_or_else(|| std::path::PathBuf::from("."))
                                .join("anarchyloader");
                            let _ = opener::open(downloads_dir);
                        }

                        if ui.cbutton("Open log file").clicked() {
                            let log_file = dirs::config_dir()
                                .unwrap_or_else(|| std::path::PathBuf::from("."))
                                .join("anarchyloader")
                                .join("anarchyloader.log");
                            let _ = opener::open(log_file);
                        }

                        let modal = Modal::new(ctx, "confirm_dialog");

                        modal.show(|ui| {
                            ui.label("Are you sure you want to reset the settings?");
                            ui.horizontal(|ui| {
                                if ui
                                    .button(RichText::new("Reset").color(egui::Color32::LIGHT_RED))
                                    .on_hover_cursor(Clickable)
                                    .clicked()
                                {
                                    self.reset_config();
                                    self.toasts.success("Settings reset.");
                                    modal.close();
                                }

                                if ui.button("Cancel").on_hover_cursor(Clickable).clicked() {
                                    modal.close();
                                }
                            });
                        });

                        if ui.button(RichText::new("Reset settings")).clicked() {
                            modal.open();
                        }
                    });
                });
        });
    }
}
