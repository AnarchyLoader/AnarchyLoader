use std::{fs, path::Path, process::Command, sync::Arc, thread, time::Duration};

use egui::{CursorIcon::PointingHand as Clickable, RichText, Spinner, TextStyle};
use egui_alignments::top_horizontal;
use egui_modal::Modal;
use is_elevated::is_elevated;

use crate::{
    custom_widgets::{Button, Hyperlink},
    default_main_menu_message,
    hacks::{self, Hack},
    MyApp,
};

impl MyApp {
    // MARK: Key events
    pub fn handle_key_events(&mut self, ctx: &egui::Context) {
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
            self.selected_hack = None;
        }

        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter)) {
            if let Some(selected) = &self.selected_hack {
                if selected.game == "CSGO" {
                    self.manual_map_injection(
                        selected.clone(),
                        ctx.clone(),
                        self.message_sender.clone(),
                    );
                } else {
                    self.start_injection(
                        selected.clone(),
                        ctx.clone(),
                        self.message_sender.clone(),
                    );
                }
            }
        }

        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::F5)) {
            self.main_menu_message = "Fetching hacks...".to_string();
            ctx.request_repaint();
            self.hacks = match hacks::Hack::fetch_hacks(
                &self.config.api_endpoint,
                self.config.lowercase_hacks,
            ) {
                Ok(hacks) => {
                    self.main_menu_message = default_main_menu_message();
                    ctx.request_repaint();
                    hacks
                }
                Err(_err) => {
                    self.main_menu_message = "Failed to fetch hacks.".to_string();
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
            ui.label("you can close this window by clicking outside of it.");
            let dropped_filename = self
                .dropped_file
                .path
                .as_ref()
                .and_then(|p| p.file_name())
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            egui::ComboBox::from_id_salt("Process")
                .selected_text(if self.selected_process_dnd.is_empty() {
                    "".to_string()
                } else {
                    self.selected_process_dnd.clone()
                })
                .show_ui(ui, |ui| {
                    for process in &self.hacks_processes {
                        ui.selectable_value(
                            &mut self.selected_process_dnd,
                            process.to_string(),
                            process.clone(),
                        )
                        .on_hover_cursor(Clickable);
                    }
                })
                .response
                .on_hover_cursor(Clickable);

            let is_csgo = self.selected_process_dnd == "csgo.exe";
            let is_cs2 = self.selected_process_dnd == "cs2.exe";

            if ui
                .cbutton(format!("Inject a {}", dropped_filename))
                .clicked()
            {
                if self.selected_process_dnd.is_empty() {
                    self.toasts.error("Please select a process.");
                    return;
                }

                self.toasts.info(format!(
                    "Injecting {}{}",
                    dropped_filename,
                    if is_csgo || is_cs2 {
                        " using manual map injection."
                    } else {
                        " using standard injection."
                    }
                ));

                if is_csgo || is_cs2 {
                    self.manual_map_inject(
                        self.dropped_file.path.clone(),
                        &self.selected_process_dnd.clone(),
                        self.message_sender.clone(),
                    );
                } else {
                    self.inject(
                        self.dropped_file.path.clone(),
                        &self.selected_process_dnd.clone(),
                        self.message_sender.clone(),
                    );
                }

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
            self.dropped_file = dropped_file.clone();
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
        let is_csgo = selected.game == "CSGO";
        let is_cs2 = selected.game == "CS2";
        top_horizontal(ui, |ui| {
            ui.horizontal(|ui| {
                ui.heading(&selected.name);
                ui.label(RichText::new(format!("by {}", selected.author)).color(theme_color));
                ui.clink("(source)", &selected.source);
            });
        });
        ui.separator();
        ui.label(&selected.description);

        if !self.config.hide_steam_account {
            ui.horizontal_wrapped(|ui| {
                let width = ui.fonts(|f| f.glyph_width(&TextStyle::Body.resolve(ui.style()), ' '));
                ui.spacing_mut().item_spacing.x = width;

                ui.label(format!("Currently logged in as (steam):"));
                ui.label(RichText::new(&self.account.name).color(theme_color))
                    .on_hover_text_at_pointer(&self.account.username)
                    .on_hover_cursor(egui::CursorIcon::Help);

                ui.label("(hover to view username)")
                    .on_hover_text_at_pointer(&self.account.username)
                    .on_hover_cursor(egui::CursorIcon::Help);
            });
        }

        // MARK: Inject button
        let is_32bit = std::mem::size_of::<usize>() == 4;
        let is_cs2_32bit = is_32bit && selected.game == "CS2";
        let inject_button = ui
            .add_enabled_ui(!is_cs2_32bit, |ui| {
                ui.button_with_tooltip(format!("Inject {}", selected.name), &selected.file)
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
                    format!("Injecting {}", selected.name),
                    "⌛".to_string(),
                    egui::Color32::from_rgb(150, 200, 210),
                )
                .duration(Some(Duration::from_secs(2)));

            self.rpc
                .update(None, Some(&format!("Injecting {}", selected.name)));

            log::info!("Injecting {}", selected.name);

            if is_csgo || is_cs2 {
                self.manual_map_injection(
                    selected.clone(),
                    ctx.clone(),
                    self.message_sender.clone(),
                );
            } else {
                self.start_injection(selected.clone(), ctx.clone(), self.message_sender.clone());
            }
        }
        if !is_elevated() && (is_csgo || is_cs2) && !self.config.hide_csgo_warning {
            ui.label(RichText::new("If you encounter an error stating that csgo.exe/cs2.exe is not found try running the loader as an administrator\nYou can disable this warning in the settings.")
                    .size(11.0)
                    .color(egui::Color32::YELLOW),
                    );
        }

        let inject_in_progress = self
            .inject_in_progress
            .load(std::sync::atomic::Ordering::SeqCst);

        if inject_in_progress {
            ui.add_space(5.0);
            let status = self.status_message.lock().unwrap().clone();
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
            let status = self.status_message.lock().unwrap().clone();
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
        let status_message = Arc::clone(&self.status_message);
        let is_favorite = self.config.favorites.contains(&hack.name);

        response.context_menu(|ui| {
            if is_favorite {
                if ui.cbutton("Remove from favorites").clicked() {
                    self.config.favorites.remove(&hack.name);
                    self.config.save_config();
                    self.toasts
                        .success(format!("Removed {} from favorites.", hack.name));
                    ui.close_menu();
                }
            } else {
                if ui.cbutton("Add to favorites").clicked() {
                    self.config.favorites.insert(hack.name.clone());
                    self.config.save_config();
                    self.toasts
                        .success(format!("Added {} to favorites.", hack.name));
                    ui.close_menu();
                }
            }

            // show only if file exists
            if Path::new(&file_path_owned).exists() {
                if ui
                    .button_with_tooltip("Open in Explorer", "Open the file location in Explorer")
                    .clicked()
                {
                    if let Err(e) = Command::new("explorer.exe")
                        .arg(format!("/select,{}", hack.file_path.to_string_lossy()))
                        .spawn()
                    {
                        let mut status = self.status_message.lock().unwrap();
                        *status = format!("Failed to open Explorer: {}", e);
                        self.toasts.error(format!("Failed to open Explorer: {}", e));
                    }
                }

                if ui
                    .button_with_tooltip("Uninstall", "Uninstall the selected hack")
                    .clicked()
                {
                    if let Err(e) = std::fs::remove_file(&file_path_owned) {
                        let mut status = self.status_message.lock().unwrap();
                        *status = format!("Failed to uninstall: {}", e);
                    } else {
                        let mut status = self.status_message.lock().unwrap();
                        *status = "Uninstall successful.".to_string();
                    }
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
                        match hack_clone.download(file_path_owned.to_string_lossy().to_string()) {
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
                }
            }
        });
    }
}
