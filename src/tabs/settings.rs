use egui::{CursorIcon::PointingHand as Clickable, RichText};
use egui_dnd::dnd;
use egui_material_icons::icons::{ICON_VISIBILITY, ICON_VISIBILITY_OFF};
use egui_modal::Modal;
use egui_theme_switch::ThemeSwitch;

#[cfg(feature = "scanner")]
use crate::scanner::scanner::ScannerPopup;
use crate::{
    games::local::{LocalHack, LocalUI},
    hacks,
    utils::{
        config::{
            default_api_endpoint, default_api_extra_endpoints, default_cdn_endpoint,
            default_cdn_extra_endpoint,
        },
        custom_widgets::{Button, CheckBox, TextEdit},
        rpc::{Rpc, RpcUpdate},
    },
    MyApp,
};

#[derive(Debug)]
pub struct TransitionPopup {
    pub duration: f32,
    pub amount: f32,
}

impl Default for TransitionPopup {
    fn default() -> Self {
        TransitionPopup {
            duration: 0.20,
            amount: 32.0,
        }
    }
}

impl MyApp {
    pub fn render_settings_tab(&mut self, ctx: &egui::Context) -> () {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .drag_to_scroll(false)
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());

                    #[cfg(feature = "scanner")]
                    {
                        ui.add_space(3.0);
                        self.render_scanner(ctx, ui);
                    }

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
                            self.app.selected_hack = None; // unselect hack because of name change
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
                            .ccheckbox(&mut self.app.config.hide_tabs_icons, "Hide tabs icons")
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

                        let modal_transition = Modal::new(ctx, "transition_configure_dialog")
                            .with_close_on_outside_click(true);

                        modal_transition.show(|ui| {
                            ui.label("Transition duration (secs):");

                            ui.add(
                                egui::Slider::new(
                                    &mut self.ui.popups.transition.duration,
                                    0.10..=1.0,
                                )
                                .text("secs"),
                            );

                            ui.label("Transition amount:");
                            ui.add(
                                egui::Slider::new(
                                    &mut self.ui.popups.transition.amount,
                                    0.0..=64.0,
                                )
                                .suffix("s")
                                .text("amount"),
                            );

                            ui.horizontal(|ui| {
                                if ui.cbutton("Confirm").clicked() {
                                    self.app.config.transition_duration =
                                        self.ui.popups.transition.duration;

                                    self.app.config.transition_amount =
                                        self.ui.popups.transition.amount;

                                    self.app.config.save();
                                    self.toasts.success("Transition updated.");
                                    modal_transition.close();
                                }
                                if ui.cbutton("Cancel").clicked() {
                                    modal_transition.close();
                                }
                            });
                        });

                        if ui
                            .ccheckbox(
                                &mut self.app.config.enable_tab_animations,
                                "Enable tab animations",
                            )
                            .changed()
                        {
                            self.app.config.save();
                        }

                        if ui.cbutton("Configure transitions").clicked() {
                            modal_transition.open();
                        }

                        ui.add_space(5.0);

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

                        let mut preference = ui.ctx().options(|opt| opt.theme_preference);
                        if ui.add(ThemeSwitch::new(&mut preference)).changed() {
                            ui.ctx().set_theme(preference);
                            self.app.config.theme = preference;
                            self.app.config.save();
                        }
                    });

                    ui.add_space(5.0);

                    // MARK: - Game order
                    ui.group(|ui| {
                        ui.label("Game Order (Drag to reorder):");
                        ui.add_space(5.0);

                        let mut game_order = self.app.config.game_order.clone();
                        let mut hidden_games = self.app.config.hidden_games.clone();
                        let response = dnd(ui, "dnd_game_order_settings").show(
                            game_order.iter_mut(),
                            |ui, game_name, handle, _| {
                                let hidden_games = &mut hidden_games;
                                handle.show_drag_cursor_on_hover(false).ui(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.label("☰").on_hover_cursor(egui::CursorIcon::Grab);
                                        if hidden_games.contains(game_name) {
                                            if ui
                                                .cbutton(ICON_VISIBILITY_OFF)
                                                .on_hover_text("Toggle on visibility")
                                                .clicked()
                                            {
                                                hidden_games.remove(game_name);
                                                self.app.config.hidden_games = hidden_games.clone();
                                                self.app.config.save();
                                            }
                                        } else {
                                            if ui
                                                .cbutton(ICON_VISIBILITY)
                                                .on_hover_text("Toggle off visibility")
                                                .clicked()
                                            {
                                                hidden_games.insert(game_name.clone());
                                                self.app.config.hidden_games = hidden_games.clone();
                                                self.app.config.save();
                                            }
                                        }
                                        ui.label(game_name.clone());
                                    });
                                });
                            },
                        );

                        if response.is_drag_finished() {
                            response.update_vec(&mut game_order);
                            self.app.config.game_order = game_order;
                            self.app.config.save();
                        }

                        ui.add_space(5.0);

                        ui.horizontal(|ui| {
                            if ui.icon_button(ICON_VISIBILITY, "Show all").clicked() {
                                self.app.config.hidden_games.clear();
                                self.app.config.save();
                            }

                            if ui.icon_button(ICON_VISIBILITY_OFF, "Hide all").clicked() {
                                self.app.config.hidden_games =
                                    self.app.config.game_order.clone().into_iter().collect();
                                self.app.config.save();
                            }
                        });

                        ui.horizontal(|ui| {
                            if ui.cbutton("Reset game order").clicked() {
                                self.app.config.reset_game_order();
                                self.toasts.success("Game order reset.");
                            }

                            if ui.cbutton(RichText::new("Reset hidden games")).clicked() {
                                self.app.config.hidden_games.clear();
                                self.app.config.save();
                                self.toasts.success("Hidden games reset.");
                            }
                        });

                        let local_hack_modal = Modal::new(ctx, "add_local_hack_modal")
                            .with_close_on_outside_click(true);
                        local_hack_modal.show(|ui| {
                            ui.label("Add Local Hack");
                            ui.separator();

                            let path_buf = &mut self.ui.popups.local_hack.new_local_dll;

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
                                &mut self.ui.popups.local_hack.new_local_process,
                            );
                            ui.label("Architecture:");
                            egui::ComboBox::from_id_salt("local_hack_arch")
                                .selected_text(&self.ui.popups.local_hack.new_local_arch)
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        &mut self.ui.popups.local_hack.new_local_arch,
                                        "x64".to_string(),
                                        "x64",
                                    )
                                    .on_hover_cursor(Clickable);
                                    ui.selectable_value(
                                        &mut self.ui.popups.local_hack.new_local_arch,
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
                                    if self.ui.popups.local_hack.new_local_dll.is_empty() {
                                        self.toasts.error("Please select a DLL file.");
                                        return;
                                    }

                                    if self.ui.popups.local_hack.new_local_process.is_empty() {
                                        self.toasts.error("Please enter a process name.");
                                        return;
                                    }

                                    if self.ui.popups.local_hack.new_local_arch.is_empty() {
                                        self.toasts.error("Please select an architecture.");
                                        return;
                                    }

                                    let hack = LocalHack {
                                        dll: self.ui.popups.local_hack.new_local_dll.clone(),
                                        process: self
                                            .ui
                                            .popups
                                            .local_hack
                                            .new_local_process
                                            .clone(),
                                        arch: self.ui.popups.local_hack.new_local_arch.clone(),
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

                                    self.ui.popups.local_hack.new_local_dll.clear();
                                    self.ui.popups.local_hack.new_local_process.clear();
                                    self.ui.popups.local_hack.new_local_arch.clear();

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
                                self.app.config.reset_game_order();
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

                        if ui.cbutton("Download stable injectors").clicked() {
                            self.download_injectors(
                                self.communication.messages.sender.clone(),
                                false,
                            );
                        }

                        if ui.cbutton("Download nightly injectors").clicked() {
                            self.download_injectors(
                                self.communication.messages.sender.clone(),
                                true,
                            );
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
                            let _ = opener::open(self.app.meta.path.clone());
                        }

                        #[cfg(debug_assertions)]
                        if ui.cbutton("Open config file").clicked() {
                            let _ = opener::open(self.app.meta.path.join("config.json").clone());
                        }
                    });

                    ui.add_space(5.0);

                    ui.horizontal(|ui| {
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
                                    self.app.config.reset_game_order();

                                    // clear popups
                                    self.ui.popups.transition = TransitionPopup::default();
                                    self.ui.popups.local_hack = LocalUI::default();

                                    #[cfg(feature = "scanner")]
                                    {
                                        self.ui.popups.scanner = ScannerPopup::default();
                                    }

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
                                    self.app.stats.reset();
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
