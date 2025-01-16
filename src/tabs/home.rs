use std::{fs, path::Path, process::Command, sync::Arc, thread, time::Duration};

use egui::{CursorIcon::PointingHand as Clickable, RichText, Spinner, TextStyle};
use egui_commonmark::CommonMarkViewer;
use egui_modal::Modal;

use crate::{
    default_main_menu_message,
    hacks::{self, Hack},
    utils::custom_widgets::{Button, CheckBox, Hyperlink},
    MyApp,
};

#[rustfmt::skip]
#[cfg(feature = "scanner")]
use crate::scanner::scanner::Scanner;

impl MyApp {
    // MARK: Key events
    pub fn handle_key_events(&mut self, ctx: &egui::Context) {
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
            self.rpc.update(None, Some("Selecting hack"), None);
            self.app.selected_hack = None;
            self.app.config.selected_hack = "".to_string();
        }

        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter)) {
            if let Some(selected) = &self.app.selected_hack {
                self.injection(
                    selected.clone(),
                    ctx.clone(),
                    self.communication.messages.sender.clone(),
                    false,
                );
            }
        }

        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::F5)) {
            self.ui.main_menu_message = "Fetching hacks...".to_string();
            ctx.request_repaint();
            self.app.hacks = match hacks::fetch_hacks(
                &self.app.config.api_endpoint,
                &self.app.config.api_extra_endpoints,
                self.app.config.lowercase_hacks,
            ) {
                Ok(hacks) => {
                    self.ui.main_menu_message = default_main_menu_message();
                    ctx.request_repaint();
                    hacks
                }
                Err(_err) => {
                    self.ui.main_menu_message = "Failed to fetch hacks.".to_string();
                    Vec::new()
                }
            };

            self.toasts.info("Hacks refreshed.");
        }
    }

    pub fn handle_dnd(&mut self, ctx: &egui::Context) {
        let modal = Modal::new(ctx, "dnd_modal").with_close_on_outside_click(true);

        modal.show(|ui| {
            ui.heading("Select process:");
            ui.add_space(5.0);
            ui.label("you can close this window by clicking outside of it.");
            ui.add_space(5.0);
            let dropped_filename = self
                .ui
                .dropped_file
                .path
                .as_ref()
                .and_then(|p| p.file_name())
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            egui::ComboBox::from_id_salt("Process")
                .selected_text(if self.ui.selected_process_dnd.is_empty() {
                    "".to_string()
                } else {
                    self.ui.selected_process_dnd.clone()
                })
                .show_ui(ui, |ui| {
                    for process in &self.app.hacks_processes {
                        ui.selectable_value(
                            &mut self.ui.selected_process_dnd,
                            process.to_string(),
                            process.clone(),
                        )
                        .on_hover_cursor(Clickable);
                    }
                })
                .response
                .on_hover_cursor(Clickable);

            ui.add_space(5.0);

            let mut force_x64 = false;

            ui.ccheckbox(&mut force_x64, "Force use x64 injector");

            ui.add_space(5.0);

            if ui
                .cbutton(format!("Inject a {}", dropped_filename))
                .clicked()
            {
                if self.ui.selected_process_dnd.is_empty() {
                    self.toasts.error("Please select a process.");
                    return;
                }

                self.toasts.info(format!(
                    "Injecting {} using manual map injection",
                    dropped_filename
                ));

                MyApp::manual_map_inject(
                    self.ui.dropped_file.path.clone(),
                    &self.ui.selected_process_dnd.clone(),
                    self.communication.messages.sender.clone(),
                    self.communication.status_message.clone(),
                    ctx.clone(),
                    force_x64,
                );
                modal.close();
            }
        });

        if let Some(dropped_file) = ctx.input(|i| i.raw.dropped_files.first().cloned()) {
            if dropped_file
                .path
                .as_ref()
                .unwrap()
                .extension()
                .unwrap_or_default()
                != "dll"
            {
                self.toasts.error("Only DLL files are supported.");
                return;
            }
            self.ui.dropped_file = dropped_file.clone();
            modal.open();
        }

        if ctx.input(|i| i.raw.hovered_files.first().is_some()) {
            let screen_rect = ctx.screen_rect();
            let painter = ctx.layer_painter(egui::LayerId::new(
                egui::Order::Foreground,
                egui::Id::new("layer"),
            ));
            painter.rect_filled(screen_rect, 0.0, egui::Color32::from_black_alpha(128));
        }
    }

    // MARK: Hack details
    pub fn display_hack_details(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        selected: &Hack,
        theme_color: egui::Color32,
    ) {
        let is_roblox = selected.game == "Roblox";

        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.heading(&selected.name);
            });
            if !selected.author.is_empty() {
                ui.horizontal_wrapped(|ui| {
                    let width =
                        ui.fonts(|f| f.glyph_width(&TextStyle::Body.resolve(ui.style()), ' '));
                    ui.spacing_mut().item_spacing.x = width;

                    ui.label("by");
                    ui.label(RichText::new(&selected.author).color(theme_color));
                    if !selected.source.is_empty() {
                        ui.clink("(source)", &selected.source);
                    }
                });
            }
        });

        ui.separator();

        if !selected.description.is_empty() {
            CommonMarkViewer::new().show(ui, &mut self.app.cache, &selected.description);
        }

        if !self.app.config.hide_steam_account && !is_roblox {
            ui.horizontal_wrapped(|ui| {
                let width = ui.fonts(|f| f.glyph_width(&TextStyle::Body.resolve(ui.style()), ' '));
                ui.spacing_mut().item_spacing.x = width;

                ui.label(format!("Currently logged in as (steam):"));
                if ui
                    .label(RichText::new(&self.app.account.name).color(theme_color))
                    .on_hover_text_at_pointer(&self.app.account.username)
                    .on_hover_cursor(egui::CursorIcon::Help)
                    .clicked()
                {
                    if let Err(e) = opener::open(format!(
                        "https://steamcommunity.com/profiles/{}/",
                        self.app.account.id
                    )) {
                        self.toasts
                            .error(format!("Failed to open Steam profile: {}", e));
                    }
                }

                ui.label("(hover to view username, click to open profile)");
            });
        }

        // MARK: Inject button
        let is_32bit = std::mem::size_of::<usize>() == 4;
        let is_cs2_32bit = is_32bit && selected.process == "cs2.exe";
        let in_progress = self
            .communication
            .in_progress
            .load(std::sync::atomic::Ordering::SeqCst);
        let inject_button = ui
            .add_enabled_ui(!in_progress && !is_cs2_32bit, |ui| {
                ui.button_with_tooltip(
                    if !is_roblox {
                        format!("Inject {}", selected.name)
                    } else {
                        "Run".to_string()
                    },
                    &selected.file,
                )
            })
            .inner;

        if is_32bit {
            ui.label(
                RichText::new("32-bit detected, cs2 hacks are not supported.")
                    .color(egui::Color32::RED),
            );
        }

        if inject_button.clicked() && !is_cs2_32bit {
            self.toasts
                .custom(
                    if !is_roblox {
                        format!("Injecting {}", selected.name)
                    } else {
                        "Running...".to_string()
                    },
                    "⌛".to_string(),
                    egui::Color32::from_rgb(150, 200, 210),
                )
                .duration(Some(Duration::from_secs(2)));

            self.rpc.update(
                None,
                Some(&if !is_roblox {
                    format!("Injecting {}", selected.name)
                } else {
                    "Running...".to_string()
                }),
                Some(if !is_roblox { "injecting" } else { "running" }),
            );

            log::info!(
                "{}",
                if !is_roblox {
                    format!("Injecting {}", selected.name)
                } else {
                    "Running...".to_string()
                }
            );

            if !is_roblox {
                self.injection(
                    selected.clone(),
                    ctx.clone(),
                    self.communication.messages.sender.clone(),
                    if ctx.input(|i| i.modifiers.ctrl) {
                        true
                    } else {
                        false
                    },
                );
            } else {
                self.run_executor(
                    selected.clone(),
                    ctx.clone(),
                    self.communication.messages.sender.clone(),
                );
            }
        }

        if in_progress {
            ui.add_space(5.0);
            let status = self.communication.status_message.lock().unwrap().clone();
            ui.horizontal(|ui| {
                ui.add(Spinner::new());
                ui.add_space(5.0);
                ui.label(
                    RichText::new(&status).color(if status.starts_with("Failed") {
                        egui::Color32::RED
                    } else {
                        theme_color
                    }),
                );
                ctx.request_repaint();
            });
        } else {
            ui.add_space(5.0);
            let status = self.communication.status_message.lock().unwrap().clone();
            if !status.is_empty() {
                let color = if status.starts_with("Failed") || status.starts_with("Error") {
                    egui::Color32::RED
                } else {
                    theme_color
                };
                ui.label(RichText::new(&status).color(color));
            }
        }
    }

    pub fn context_menu(&mut self, response: &egui::Response, ctx: &egui::Context, hack: &Hack) {
        // MARK: Context menu
        let file_path_owned = hack.file_path.clone();
        let ctx_clone = ctx.clone();
        let status_message = Arc::clone(&self.communication.status_message);
        let is_favorite = self.app.config.favorites.contains(&hack.name);
        let is_roblox = hack.game == "Roblox";

        response.context_menu(|ui| {
            if is_favorite {
                if ui.cbutton("Remove from favorites").clicked() {
                    self.app.config.favorites.remove(&hack.name);
                    self.app.config.save();
                    self.toasts
                        .success(format!("Removed {} from favorites.", hack.name));
                    ui.close_menu();
                }
            } else {
                if ui.cbutton("Add to favorites").clicked() {
                    self.app.config.favorites.insert(hack.name.clone());
                    self.app.config.save();
                    self.toasts
                        .success(format!("Added {} to favorites.", hack.name));
                    ui.close_menu();
                }
            }

            if !is_roblox && !hack.local {
                // show only if file exists
                if Path::new(&file_path_owned).exists() {
                    if ui
                        .button_with_tooltip(
                            "Open in Explorer",
                            "Open the file location in Explorer",
                        )
                        .clicked()
                    {
                        if let Err(e) = Command::new("explorer.exe")
                            .arg(format!("/select,{}", hack.file_path.to_string_lossy()))
                            .spawn()
                        {
                            let mut status = self.communication.status_message.lock().unwrap();
                            *status = format!("Failed to open Explorer: {}", e);
                            self.toasts.error(format!("Failed to open Explorer: {}", e));
                        }
                        ui.close_menu();
                    }

                    if ui
                        .button_with_tooltip("Uninstall", "Uninstall the selected hack")
                        .clicked()
                    {
                        if let Err(e) = std::fs::remove_file(&file_path_owned) {
                            let mut status = self.communication.status_message.lock().unwrap();
                            *status = format!("Failed to uninstall: {}", e);
                        } else {
                            let mut status = self.communication.status_message.lock().unwrap();
                            *status = "Uninstall successful.".to_string();
                        }
                        ui.close_menu();
                    }

                    if ui
                        .button_with_tooltip("Reinstall", "Reinstall the selected hack")
                        .clicked()
                    {
                        let hack_clone = hack.clone();
                        thread::spawn(move || {
                            if !Path::new(&file_path_owned).exists() {
                                let mut status = status_message.lock().unwrap();
                                *status = "Failed to reinstall: file does not exist.".to_string();
                                ctx_clone.request_repaint();
                                return;
                            }
                            {
                                let mut status = status_message.lock().unwrap();
                                *status = "Reinstalling...".to_string();
                                ctx_clone.request_repaint();
                            }
                            if let Err(e) = fs::remove_file(&file_path_owned) {
                                let mut status = status_message.lock().unwrap();
                                *status = format!("Failed to delete file: {}", e);
                                ctx_clone.request_repaint();
                                return;
                            }
                            match hack_clone.download(file_path_owned.to_string_lossy().to_string())
                            {
                                Ok(_) => {
                                    let mut status = status_message.lock().unwrap();
                                    *status = "Reinstalled.".to_string();
                                    ctx_clone.request_repaint();
                                }
                                Err(e) => {
                                    let mut status = status_message.lock().unwrap();
                                    *status = format!("Failed to reinstall: {}", e);
                                    ctx_clone.request_repaint();
                                }
                            }
                        });
                        ui.close_menu();
                    }

                    #[cfg(feature = "scanner")]
                    if ui
                        .button_with_tooltip("Scan", "Scan the selected hack")
                        .clicked()
                    {
                        let scanner =
                            Scanner::new(std::path::PathBuf::from(hack.file_path.clone()));

                        match scanner.scan(self.app_path.clone()) {
                            Ok(()) => {
                                match opener::open(self.app_path.join("scanner_results.txt")) {
                                    Ok(_) => {
                                        self.toasts.success("Results opened.");
                                    }
                                    Err(err) => {
                                        self.toasts
                                            .error(format!("Failed to open results: {}", err));
                                    }
                                }
                            }
                            Err(err) => {
                                self.toasts.error(err);
                            }
                        }
                        ui.close_menu();
                    }
                } else {
                    if ui.cbutton("Download").clicked() {
                        let file_path = hack.file_path.clone();
                        let hack_clone = hack.clone();
                        thread::spawn(move || {
                            match hack_clone.download(file_path.to_string_lossy().to_string()) {
                                Ok(_) => {
                                    let mut status = status_message.lock().unwrap();
                                    *status = "Downloaded.".to_string();
                                }
                                Err(e) => {
                                    let mut status = status_message.lock().unwrap();
                                    *status = format!("Failed to download: {}", e);
                                }
                            }
                        });
                        ui.close_menu();
                    }
                }
            }

            if hack.local {
                if ui.cbutton("Remove").clicked() {
                    self.app.config.local_hacks.retain(|h| {
                        Path::new(&h.dll)
                            .file_name()
                            .map_or(true, |f| f != hack.file_path.file_name().unwrap())
                    });
                    self.app.config.save();
                    let grouped =
                        MyApp::group_hacks_by_game_internal(&self.app.hacks, &self.app.config);
                    self.app.config.game_order = grouped.keys().cloned().collect();
                    self.toasts.success(format!("Removed {}.", hack.name));
                    ui.close_menu();
                }
            }
        });
    }
}
