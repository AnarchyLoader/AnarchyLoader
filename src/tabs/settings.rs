use egui::{CursorIcon::PointingHand as Clickable, RichText, ThemePreference};
use egui_dnd::dnd;
use egui_material_icons::icons::{
    ICON_ADD, ICON_CLOSE, ICON_DELETE, ICON_DOWNLOAD, ICON_EYE_TRACKING, ICON_FOLDER,
    ICON_MANUFACTURING, ICON_RESTART_ALT, ICON_VISIBILITY, ICON_VISIBILITY_OFF,
};
use egui_modal::Modal;
use egui_theme_switch::ThemeSwitch;

#[cfg(feature = "scanner")]
use crate::scanner::scanner::ScannerPopup;
use crate::{
    games::local::{LocalHack, LocalUI},
    utils::{
        api::{
            api_settings::{
                default_api_endpoint, default_api_extra_endpoints, default_cdn_endpoint,
                default_cdn_extra_endpoints,
            },
            hacks,
        },
        rpc::{Rpc, RpcUpdate},
        ui::widgets::{Button, CheckBox, TextEdit},
    },
    MyApp,
};

impl MyApp {
    pub fn render_settings_tab(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .drag_to_scroll(false)
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());

                    #[cfg(feature = "scanner")]
                    {
                        // MARK: - Scanner
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
                            MyApp::group_hacks_by_game_internal(&self.app.hacks, &self.app.config);
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
                                &self.app.config.api.api_endpoint,
                                &self.app.config.api.api_extra_endpoints,
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
                            .ccheckbox(&mut self.app.config.display.disable_hack_name_animation, "Disable hack name animation")
                            .changed()
                        {
                            self.app.config.save();
                        }
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
                                log::info!("<SETTINGS_TAB> Discord RPC enabled");
                            } else {
                                self.rpc.sender.send(RpcUpdate::Shutdown).ok();
                                log::info!("<SETTINGS_TAB> Discord RPC disabled");
                            }
                        }
                        if ui
                            .ccheckbox(
                                &mut self.app.config.display.hide_steam_account,
                                "Hide Steam account",
                            )
                            .changed()
                        {
                            self.app.config.save();
                        }
                        if ui
                            .ccheckbox(&mut self.app.config.display.hide_tabs_icons, "Hide tabs icons")
                            .changed()
                        {
                            self.app.config.save();
                        }
                        if ui
                            .ccheckbox(&mut self.app.config.display.hide_statistics, "Hide statistics")
                            .changed()
                        {
                            self.app.config.save();
                        };
                        if ui
                            .ccheckbox(
                                &mut self.app.config.display.disable_toasts,
                                "Disable toasts",
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
                            .ccheckbox(&mut self.app.config.display.skip_update_check, "Skip update check")
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

                        let modal_animations = Modal::new(ctx, "animations_configure_dialog")
                            .with_close_on_outside_click(true);

                        modal_animations.show(|ui| {
                            ui.label("Transition duration:");

                            if ui.add(
                                egui::Slider::new(
                                    &mut self.app.config.animations.duration,
                                    0.10..=1.0,
                                )
                                    .text("secs"),
                            ).changed() {
                                self.app.config.save()
                            };

                            ui.label("Transition amount:");
                            if ui.add(
                                egui::Slider::new(
                                    &mut self.app.config.animations.amount,
                                    0.0..=128.0,
                                )
                                    .suffix("px")
                                    .text("pixels"),
                            ).changed() {
                                self.app.config.save()
                            };

                            ui.label("Text animation speed:");
                            if ui.add(
                                egui::Slider::new(
                                    &mut self.ui.text_animator.speed,
                                    0.0..=3.5,
                                )
                                    .suffix("x"),
                            ).changed() {
                                self.app.config.animations.text_speed = self.ui.text_animator.speed;
                                self.app.config.save()
                            };

                            ui.add_space(5.0);

                            ui.horizontal(|ui| {
                                if ui
                                    .reset_button("Reset")
                                    .clicked()
                                {
                                    self.app.config.animations = Default::default();
                                    self.app.config.save();
                                    self.ui.text_animator.speed = self.app.config.animations.text_speed;
                                    log::info!("<SETTINGS_TAB> Transition settings reset to default, saving config");
                                }

                                if ui.cibutton("Close", ICON_CLOSE).clicked() {
                                    modal_animations.close();
                                }
                            });
                        });

                        if ui
                            .ccheckbox(
                                &mut self.app.config.animations.tab_animations,
                                "Enable tab animations",
                            )
                            .changed()
                        {
                            self.app.config.save();
                        }

                        ui.add_space(3.0);

                        if ui
                            .cbutton(format!("{} Configure animations", ICON_MANUFACTURING))
                            .clicked()
                        {
                            modal_animations.open();
                        }

                        ui.add_space(5.0);

                        ui.horizontal(|ui| {
                            ui.label("Favorites Color:");
                            if ui
                                .color_edit_button_srgba(&mut self.app.config.display.favorites_color)
                                .on_hover_cursor(Clickable)
                                .changed()
                            {
                                self.app.config.save();
                            }
                        });

                        ui.add_space(5.0);

                        let mut preference = ui.ctx().options(|opt| opt.theme_preference);

                        if ui.add(ThemeSwitch::new(&mut preference)).changed() {
                            ui.ctx().set_theme(preference);
                            self.app.config.display.theme = preference;
                            self.app.config.save();
                            log::info!("<SETTINGS_TAB> Theme preference changed to: {:?}, saving config", preference);

                            if preference == ThemePreference::Light {
                                self.toasts.info("Dark theme is recommended for a better experience.");
                            }
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
                                                log::info!("<SETTINGS_TAB> Game '{}' visibility toggled on, saving config", game_name);
                                            }
                                        } else if ui
                                            .cbutton(ICON_VISIBILITY)
                                            .on_hover_text("Toggle off visibility")
                                            .clicked()
                                        {
                                            hidden_games.insert(game_name.clone());
                                            self.app.config.hidden_games = hidden_games.clone();
                                            self.app.config.save();
                                            log::info!("<SETTINGS_TAB> Game '{}' visibility toggled off, saving config", game_name);
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
                            log::info!("<SETTINGS_TAB> Game order updated and saved: {:?}", self.app.config.game_order);
                        }

                        ui.add_space(5.0);

                        ui.horizontal(|ui| {
                            if ui.cibutton("Show all", ICON_VISIBILITY).clicked() {
                                self.app.config.hidden_games.clear();
                                self.app.config.save();
                                log::info!("<SETTINGS_TAB> All games set to visible, saving config");
                            }

                            if ui.cibutton("Hide all", ICON_VISIBILITY_OFF).clicked() {
                                self.app.config.hidden_games =
                                    self.app.config.game_order.clone().into_iter().collect();
                                self.app.config.save();
                                log::info!("<SETTINGS_TAB> All games set to hidden, saving config");
                            }
                        });

                        ui.horizontal(|ui| {
                            if ui
                                .reset_button("Reset game order")
                                .clicked()
                            {
                                self.app.config.reset_game_order();
                                self.toasts.success("Game order reset.");
                                log::info!("<SETTINGS_TAB> Game order reset to default.");
                            }

                            if ui
                                .cbutton(RichText::new(format!(
                                    "{} Reset hidden games",
                                    ICON_EYE_TRACKING
                                )))
                                .clicked()
                            {
                                self.app.config.hidden_games.clear();
                                self.app.config.save();
                                self.toasts.success("Hidden games reset.");
                                log::info!("<SETTINGS_TAB> Hidden games reset to default, saving config");
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
                                        log::info!("<SETTINGS_TAB> DLL file selected for local hack: {}", path_buf);
                                    } else {
                                        self.toasts.error("Please select a DLL file.");
                                        log::warn!("<SETTINGS_TAB> User selected a non-DLL file for local hack: {}", path_buf);
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
                                if ui.confirm_button().clicked() {
                                    if self.ui.popups.local_hack.new_local_dll.is_empty() {
                                        self.toasts.error("Please select a DLL file.");
                                        log::warn!("<SETTINGS_TAB> Attempted to add local hack without DLL file selected.");
                                        return;
                                    }

                                    if self.ui.popups.local_hack.new_local_process.is_empty() {
                                        self.toasts.error("Please enter a process name.");
                                        log::warn!("<SETTINGS_TAB> Attempted to add local hack without process name.");
                                        return;
                                    }

                                    if self.ui.popups.local_hack.new_local_arch.is_empty() {
                                        self.toasts.error("Please select an architecture.");
                                        log::warn!("<SETTINGS_TAB> Attempted to add local hack without architecture selected.");
                                        return;
                                    }

                                    let hack = LocalHack::new(self.ui.popups.local_hack.new_local_dll.clone(), self.ui.popups.local_hack.new_local_process.clone(), self.ui.popups.local_hack.new_local_arch.clone());

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
                                    log::info!("<SETTINGS_TAB> Local hack added successfully.");
                                }
                                if ui.cibutton("Cancel", ICON_CLOSE).clicked() {
                                    local_hack_modal.close();
                                }
                            });
                        });

                        ui.horizontal(|ui| {
                            if ui.cibutton("Add local hack", ICON_ADD).clicked() {
                                local_hack_modal.open();
                            }
                            if ui.cibutton("Reset local hacks", ICON_DELETE).clicked() {
                                self.app.config.local_hacks.clear();
                                self.app.config.reset_game_order();
                                self.app.config.save();
                                self.toasts.success("Local hacks reset.");
                                log::info!("<SETTINGS_TAB> Local hacks reset to default.");
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
                                        self.toasts.error(err.clone());
                                        log::error!("<SETTINGS_TAB> Failed to delete x64 injector: {}", err);
                                    } else {
                                        self.toasts.success("x64 injector deleted.");
                                        modal_injector.close();
                                        log::info!("<SETTINGS_TAB> x64 injector deleted successfully.");
                                    }
                                }

                                if ui
                                    .cbutton(RichText::new("x86").color(egui::Color32::LIGHT_RED))
                                    .clicked()
                                {
                                    if let Err(err) = self.delete_injectors("x86") {
                                        self.toasts.error(err.clone());
                                        log::error!("<SETTINGS_TAB> Failed to delete x86 injector: {}", err);
                                    } else {
                                        self.toasts.success("x86 injector deleted.");
                                        modal_injector.close();
                                        log::info!("<SETTINGS_TAB> x86 injector deleted successfully.");
                                    }
                                    modal_injector.close();
                                }

                                if ui
                                    .cbutton(RichText::new("Both").color(egui::Color32::LIGHT_RED))
                                    .clicked()
                                {
                                    if let Err(err) = self.delete_injectors("both") {
                                        self.toasts.error(err.clone());
                                    } else {
                                        self.toasts.success("Both injectors deleted.");
                                        modal_injector.close();
                                    }
                                }

                                if ui.cibutton("Cancel", ICON_CLOSE).clicked() {
                                    modal_injector.close();
                                }
                            });
                        });

                        if ui.cibutton("Delete injector", ICON_DELETE).clicked() {
                            modal_injector.open();
                        }

                        if ui
                            .cibutton("Download stable injectors", ICON_DOWNLOAD)
                            .clicked()
                        {
                            self.download_injectors(
                                self.communication.messages.sender.clone(),
                                false,
                            );
                        }

                        if ui
                            .cibutton("Download nightly injectors", ICON_DOWNLOAD)
                            .clicked()
                        {
                            self.download_injectors(
                                self.communication.messages.sender.clone(),
                                true,
                            );
                        }
                    });

                    ui.add_space(5.0);
                    ui.group(|ui| {
                        ui.label("Right-click the input field to reset these text settings.");

                        ui.add_space(2.0);

                        ui.horizontal(|ui| {
                            ui.label("API Endpoint:");
                            if ui
                                .ctext_edit(&mut self.app.config.api.api_endpoint, default_api_endpoint())
                                .changed()
                            {
                                self.app.config.save();
                            }
                        });

                        ui.add_space(2.0);

                        ui.horizontal(|ui| {
                            ui.label("API Extra Endpoints (comma-separated):");
                            let mut api_extra_endpoints = self.app.config.api.api_extra_endpoints.join(",");
                            if ui.ctext_edit(&mut api_extra_endpoints, default_api_extra_endpoints().join(",")).changed() {
                                self.app.config.api.api_extra_endpoints = api_extra_endpoints
                                    .split(',')
                                    .map(|s| s.trim().to_string())
                                    .collect();
                                self.app.config.save();
                            }
                        });

                        ui.add_space(2.0);

                        ui.horizontal(|ui| {
                            ui.label("CDN Endpoint:");
                            if ui
                                .ctext_edit(&mut self.app.config.api.cdn_endpoint, default_cdn_endpoint())
                                .changed()
                            {
                                self.app.config.save();
                            }
                        });

                        ui.add_space(2.0);

                        ui.horizontal(|ui| {
                            ui.label("CDN Extra Endpoints (comma-separated):");
                            let mut cdn_extra_endpoints = self.app.config.api.cdn_extra_endpoints.join(",");
                            if ui.ctext_edit(&mut cdn_extra_endpoints, default_cdn_extra_endpoints().join(",")).changed() {
                                self.app.config.api.cdn_extra_endpoints = cdn_extra_endpoints
                                    .split(',')
                                    .map(|s| s.trim().to_string())
                                    .collect();
                                self.app.config.save();
                            }
                        });
                    });

                    ui.add_space(5.0);

                    ui.horizontal(|ui| {
                        if ui.cibutton("Open loader folder", ICON_FOLDER).clicked() {
                            let _ = opener::open(self.app.meta.path.clone());
                            log::info!("<SETTINGS_TAB> Opened loader folder: {}", self.app.meta.path.display());
                        }

                        #[cfg(debug_assertions)]
                        if ui.cibutton("Open config file", ICON_MANUFACTURING).clicked() {
                            let _ = opener::open(self.app.meta.path.join("config.json").clone());
                            log::info!("<SETTINGS_TAB> Opened config file: {}", self.app.meta.path.join("config.json").display());
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
                                    .confirm_button()
                                    .clicked()
                                {
                                    self.app.config.reset();
                                    self.app.config.reset_game_order();

                                    // clear popups
                                    self.ui.popups.local_hack = LocalUI::default();

                                    #[cfg(feature = "scanner")]
                                    {
                                        self.ui.popups.scanner = ScannerPopup::default();
                                    }

                                    self.toasts.success("Settings reset.");
                                    modal_settings.close();
                                    log::info!("<SETTINGS_TAB> Settings reset to default.");
                                }

                                if ui.cibutton("Cancel", ICON_CLOSE).clicked() {
                                    modal_settings.close();
                                }
                            });
                        });

                        if ui
                            .cbutton(
                                RichText::new(format!("{} Reset settings", ICON_RESTART_ALT)).color(egui::Color32::LIGHT_RED),
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
                                    .confirm_button()
                                    .clicked()
                                {
                                    self.app.stats.reset();
                                    self.toasts.success("Statistics reset.");
                                    modal_statistics.close();
                                    log::info!("<SETTINGS_TAB> Statistics reset to default.");
                                }

                                if ui.cibutton("Cancel", ICON_CLOSE).clicked() {
                                    modal_statistics.close();
                                }
                            });
                        });

                        if ui.reset_button("Reset statistics").clicked() {
                            modal_statistics.open();
                        }
                    });
                });
        });
    }
}
