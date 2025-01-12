use egui::{CursorIcon::PointingHand as Clickable, RichText};
use egui_dnd::dnd;
use egui_modal::Modal;

use crate::{
    custom_widgets::{Button, CheckBox, TextEdit},
    games::local::LocalHack,
    hacks,
    utils::{
        config::{
            default_api_endpoint, default_api_extra_endpoints, default_cdn_endpoint,
            default_cdn_extra_endpoint,
        },
        rpc::{Rpc, RpcUpdate},
    },
    MyApp,
};

impl MyApp {
    pub fn render_settings_tab(&mut self, ctx: &egui::Context) -> () {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .drag_to_scroll(false)
                .show(ui, |ui| {
                    ui.heading("Settings");
                    ui.separator();

                    ui.add_space(5.0);

                    // MARK: - Display Options
                    ui.group(|ui| {
                        ui.label("Display Options:");
                        ui.add_space(5.0);

                        if ui
                            .ccheckbox(
                                &mut self.app.config.show_only_favorites,
                                "Show only favorite hacks",
                            )
                            .changed()
                        {
                            self.app.config.save();
                        }
                        if ui
                            .ccheckbox(
                                &mut self.app.config.lowercase_hacks,
                                "Lowercase hack names & descriptions",
                            )
                            .changed()
                        {
                            self.app.hacks = match hacks::fetch_hacks(
                                &self.app.config.api_endpoint,
                                &self.app.config.api_extra_endpoints,
                                self.app.config.lowercase_hacks,
                            ) {
                                Ok(hacks) => hacks,
                                Err(_err) => {
                                    self.ui.main_menu_message =
                                        "Failed to fetch hacks.".to_string();
                                    Vec::new()
                                }
                            };

                            self.toasts.info(format!(
                                "Hacks refreshed{}.",
                                if self.app.config.lowercase_hacks {
                                    " (with lowercase)"
                                } else {
                                    ""
                                }
                            ));
                            self.app.config.save();
                        };
                        if ui
                            .ccheckbox(&mut self.app.config.disable_rpc, "Disable RPC")
                            .changed()
                        {
                            self.app.config.save();
                            if !self.app.config.disable_rpc {
                                self.rpc = Rpc::new(true);
                                self.rpc.update(
                                    Some(&format!("v{}", env!("CARGO_PKG_VERSION"))),
                                    Some("Configuring settings"),
                                    None,
                                );
                            } else {
                                self.rpc.sender.send(RpcUpdate::Shutdown).ok();
                            }
                        }
                        if ui
                            .ccheckbox(
                                &mut self.app.config.hide_steam_account,
                                "Hide Steam account",
                            )
                            .changed()
                        {
                            self.app.config.save();
                        }
                        if ui
                            .ccheckbox(&mut self.app.config.hide_statistics, "Hide statistics")
                            .changed()
                        {
                            self.app.config.save();
                        };
                        if ui
                            .ccheckbox(
                                &mut self.app.config.disable_notifications,
                                "Disable notifications",
                            )
                            .changed()
                        {
                            self.app.config.save();
                        }
                        if ui
                            .ccheckbox(
                                &mut self.app.config.skip_injects_delay,
                                "Skip injects delay (visual)",
                            )
                            .changed()
                        {
                            self.app.config.save();
                        }
                        if ui
                            .ccheckbox(&mut self.app.config.skip_update_check, "Skip update check")
                            .changed()
                        {
                            self.app.config.save();
                        }
                        if ui
                            .ccheckbox(
                                &mut self.app.config.automatically_select_hack,
                                "Automatically select recently hack",
                            )
                            .changed()
                        {
                            self.app.config.save();
                        }

                        ui.horizontal(|ui| {
                            ui.label("Favorites Color:");
                            if ui
                                .color_edit_button_srgba(&mut self.app.config.favorites_color)
                                .on_hover_cursor(Clickable)
                                .changed()
                            {
                                self.app.config.save();
                            }
                        });
                    });

                    ui.add_space(5.0);

                    ui.group(|ui| {
                        ui.label("Game Order (Drag to reorder):");
                        ui.add_space(5.0);

                        let game_order = &mut self.app.config.game_order;
                        let response = dnd(ui, "dnd_game_order_settings").show(
                            game_order.iter_mut(),
                            |ui, game_name, handle, _| {
                                handle.ui(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.label("☰");
                                        ui.label(game_name.clone());
                                    });
                                });
                            },
                        );

                        if response.is_drag_finished() {
                            response.update_vec(game_order);
                            self.app.config.save();
                        }

                        ui.add_space(5.0);

                        if ui.cbutton(RichText::new("Reset game order")).clicked() {
                            self.app.config.reset_game_order();
                            self.toasts.success("Game order reset.");
                        }

                        ui.add_space(5.0);

                        let local_hack_modal = Modal::new(ctx, "add_local_hack_modal")
                            .with_close_on_outside_click(true);
                        local_hack_modal.show(|ui| {
                            ui.label("Add Local Hack");
                            ui.separator();

                            let path_buf = &mut self.ui.local_hack_popup.new_local_dll;

                            ui.label(if path_buf.is_empty() {
                                "DLL:".to_string()
                            } else {
                                format!("DLL: {}", path_buf)
                            });

                            if ui.cbutton("Browse").clicked() {
                                if let Some(path) = rfd::FileDialog::new()
                                    .add_filter("DLL files", &["dll"])
                                    .pick_file()
                                {
                                    *path_buf = path.to_string_lossy().into_owned();
                                    if path_buf.ends_with(".dll") {
                                        self.toasts.success("DLL selected.");
                                    } else {
                                        self.toasts.error("Please select a DLL file.");
                                    }
                                }
                            }

                            ui.label("Process:");
                            ui.text_edit_singleline(
                                &mut self.ui.local_hack_popup.new_local_process,
                            );
                            ui.label("Architecture:");
                            egui::ComboBox::from_id_salt("local_hack_arch")
                                .selected_text(&self.ui.local_hack_popup.new_local_arch)
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        &mut self.ui.local_hack_popup.new_local_arch,
                                        "x64".to_string(),
                                        "x64",
                                    )
                                    .on_hover_cursor(Clickable);
                                    ui.selectable_value(
                                        &mut self.ui.local_hack_popup.new_local_arch,
                                        "x86".to_string(),
                                        "x86",
                                    )
                                    .on_hover_cursor(Clickable);
                                })
                                .response
                                .on_hover_cursor(Clickable);

                            ui.add_space(5.0);

                            ui.horizontal(|ui| {
                                if ui.cbutton("Confirm").clicked() {
                                    if self.ui.local_hack_popup.new_local_dll.is_empty() {
                                        self.toasts.error("Please select a DLL file.");
                                        return;
                                    }

                                    if self.ui.local_hack_popup.new_local_process.is_empty() {
                                        self.toasts.error("Please enter a process name.");
                                        return;
                                    }

                                    if self.ui.local_hack_popup.new_local_arch.is_empty() {
                                        self.toasts.error("Please select an architecture.");
                                        return;
                                    }

                                    let hack = LocalHack {
                                        dll: self.ui.local_hack_popup.new_local_dll.clone(),
                                        process: self.ui.local_hack_popup.new_local_process.clone(),
                                        arch: self.ui.local_hack_popup.new_local_arch.clone(),
                                    };
                                    self.add_local_hack(hack);
                                    if !self.app.config.local_hacks.is_empty()
                                        && !self
                                            .app
                                            .config
                                            .game_order
                                            .contains(&"Added".to_string())
                                    {
                                        self.app.config.game_order.push("Added".to_string());
                                    }
                                    MyApp::group_hacks_by_game_internal(
                                        &self.app.hacks,
                                        &self.app.config,
                                    );

                                    self.ui.local_hack_popup.new_local_dll.clear();
                                    self.ui.local_hack_popup.new_local_process.clear();
                                    self.ui.local_hack_popup.new_local_arch.clear();

                                    self.toasts.success("Local hack added.");
                                    local_hack_modal.close();
                                }
                                if ui.cbutton("Cancel").clicked() {
                                    local_hack_modal.close();
                                }
                            });
                        });

                        ui.horizontal(|ui| {
                            if ui.cbutton("Add local hack").clicked() {
                                local_hack_modal.open();
                            }
                            if ui.cbutton("Reset local hacks").clicked() {
                                self.app.config.local_hacks.clear();
                                self.app.config.save();
                                self.toasts.success("Local hacks reset.");
                            }
                        });
                    });

                    ui.add_space(5.0);

                    // MARK: - Injection/Delay Options
                    ui.group(|ui| {
                        ui.label("Injection Options:");

                        let modal_injector = Modal::new(ctx, "injector_confirm_dialog")
                            .with_close_on_outside_click(true);

                        modal_injector.show(|ui| {
                            ui.label("Select architecture to delete:");
                            ui.horizontal(|ui| {
                                if ui
                                    .cbutton(RichText::new("x64").color(egui::Color32::LIGHT_RED))
                                    .clicked()
                                {
                                    if let Err(err) = self.delete_injectors("x64") {
                                        self.toasts.error(err);
                                    } else {
                                        self.toasts.success("x64 injector deleted.");
                                        modal_injector.close();
                                    }
                                }

                                if ui
                                    .cbutton(RichText::new("x86").color(egui::Color32::LIGHT_RED))
                                    .clicked()
                                {
                                    if let Err(err) = self.delete_injectors("x86") {
                                        self.toasts.error(err);
                                    } else {
                                        self.toasts.success("x86 injector deleted.");
                                        modal_injector.close();
                                    }
                                    modal_injector.close();
                                }

                                if ui
                                    .cbutton(RichText::new("Both").color(egui::Color32::LIGHT_RED))
                                    .clicked()
                                {
                                    if let Err(err) = self.delete_injectors("both") {
                                        self.toasts.error(err);
                                    } else {
                                        self.toasts.success("Both injectors deleted.");
                                        modal_injector.close();
                                    }
                                }

                                if ui.cbutton("Cancel").clicked() {
                                    modal_injector.close();
                                }
                            });
                        });

                        if ui.cbutton("Delete injector").clicked() {
                            modal_injector.open();
                        }
                        if ui.cbutton("Download nightly injectors").clicked() {
                            if let Err(err) = self.download_injectors() {
                                self.toasts.error(err);
                            } else {
                                self.toasts.success("Nightly injectors downloaded.");
                            }
                        }
                    });

                    ui.add_space(5.0);

                    ui.label("Right-click the input field to reset these text settings.");

                    ui.add_space(2.0);

                    ui.horizontal(|ui| {
                        ui.label("API Endpoint:");
                        if ui
                            .ctext_edit(&mut self.app.config.api_endpoint, default_api_endpoint())
                            .changed()
                        {
                            self.app.config.save();
                        }
                    });

                    ui.add_space(2.0);

                    ui.horizontal(|ui| {
                        ui.label("API Extra Endpoints (comma-separated):");
                        if ui
                            .ctext_edit(
                                &mut self.app.config.api_extra_endpoints.join(","),
                                default_api_extra_endpoints().join(","),
                            )
                            .changed()
                        {
                            self.app.config.save();
                        }
                    });

                    ui.add_space(2.0);

                    ui.horizontal(|ui| {
                        ui.label("CDN Endpoint:");
                        if ui
                            .ctext_edit(&mut self.app.config.cdn_endpoint, default_cdn_endpoint())
                            .changed()
                        {
                            self.app.config.save();
                        }
                    });

                    ui.add_space(2.0);

                    ui.horizontal(|ui| {
                        ui.label("CDN Extra Endpoints (comma-separated):");
                        if ui
                            .ctext_edit(
                                &mut self.app.config.cdn_extra_endpoints.join(","),
                                default_cdn_extra_endpoint().join(","),
                            )
                            .changed()
                        {
                            self.app.config.save();
                        }
                    });

                    ui.add_space(5.0);

                    ui.horizontal(|ui| {
                        if ui.cbutton("Open loader folder").clicked() {
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

                        let modal_settings = Modal::new(ctx, "settings_reset_confirm_dialog")
                            .with_close_on_outside_click(true);

                        modal_settings.show(|ui| {
                            ui.label("Are you sure you want to reset the settings?");
                            ui.horizontal(|ui| {
                                if ui
                                    .cbutton(RichText::new("Reset").color(egui::Color32::LIGHT_RED))
                                    .clicked()
                                {
                                    self.app.config.reset();
                                    self.toasts.success("Settings reset.");
                                    modal_settings.close();
                                }

                                if ui.cbutton("Cancel").clicked() {
                                    modal_settings.close();
                                }
                            });
                        });

                        if ui
                            .cbutton(
                                RichText::new("Reset settings").color(egui::Color32::LIGHT_RED),
                            )
                            .clicked()
                        {
                            modal_settings.open();
                        }

                        let modal_statistics = Modal::new(ctx, "statistics_reset_confirm_dialog")
                            .with_close_on_outside_click(true);

                        modal_statistics.show(|ui| {
                            ui.label("Are you sure you want to reset the statistics?");
                            ui.horizontal(|ui| {
                                if ui
                                    .cbutton(RichText::new("Reset").color(egui::Color32::LIGHT_RED))
                                    .clicked()
                                {
                                    self.app.statistics.reset();
                                    self.toasts.success("Statistics reset.");
                                    modal_statistics.close();
                                }

                                if ui.cbutton("Cancel").clicked() {
                                    modal_statistics.close();
                                }
                            });
                        });

                        if ui.cbutton(RichText::new("Reset statistics")).clicked() {
                            modal_statistics.open();
                        }
                    });
                });
        });
    }
}
