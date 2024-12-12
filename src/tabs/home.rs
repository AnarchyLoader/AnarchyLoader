use std::{fs, path::Path, process::Command, sync::Arc, thread, time::Duration};

use egui::{CursorIcon::PointingHand as Clickable, RichText, Spinner};
use is_elevated::is_elevated;

use crate::{
    custom_widgets::Button,
    hacks::{self, Hack},
    MyApp,
};

impl MyApp {
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
            self.hacks = match hacks::Hack::fetch_hacks(&self.config.api_endpoint) {
                Ok(hacks) => {
                    self.main_menu_message = "Please select a cheat from the list.".to_string();
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

    pub fn display_hack_details(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        selected: &Hack,
        theme_color: egui::Color32,
    ) {
        let is_csgo = selected.game == "CSGO";

        ui.horizontal(|ui| {
            ui.heading(&selected.name);
            ui.label(RichText::new(format!("by {}", selected.author)).color(theme_color));
            ui.hyperlink_to("(source)", &selected.source)
                .on_hover_text(&selected.source)
        });
        ui.separator();
        ui.label(&selected.description);

        ui.add_space(5.0);
        // MARK: Inject button
        if ui
            .button(format!("Inject {}", selected.name))
            .on_hover_cursor(Clickable)
            .on_hover_text(&selected.file)
            .clicked()
        {
            self.toasts
                .custom(
                    format!("Injecting {}", selected.name),
                    "⌛".to_string(),
                    egui::Color32::from_rgb(150, 200, 210),
                )
                .duration(Some(Duration::from_secs(2)));
            if is_csgo {
                self.manual_map_injection(
                    selected.clone(),
                    ctx.clone(),
                    self.message_sender.clone(),
                );
            } else {
                self.start_injection(selected.clone(), ctx.clone(), self.message_sender.clone());
            }
        }
        if !is_elevated() && is_csgo && !self.config.hide_csgo_warning {
            ui.label(
                        RichText::new("If you encounter an error stating that csgo.exe is not found try running the loader as an administrator\nYou can disable this warning in the settings.")
                            .size(11.0)
                            .color(egui::Color32::YELLOW),
                    );
        }

        let inject_in_progress = self
            .inject_in_progress
            .load(std::sync::atomic::Ordering::SeqCst);

        if inject_in_progress {
            ui.add_space(10.0);
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
            ui.add_space(10.0);
            let status = self.status_message.lock().unwrap().clone();
            if !status.is_empty() {
                let color = if status.starts_with("Failed") {
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
