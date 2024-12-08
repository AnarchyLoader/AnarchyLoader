#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod custom_widgets;
mod downloader;
mod hacks;
mod inject;

use std::{
    collections::BTreeMap,
    env, fs,
    path::Path,
    process::Command,
    sync::{Arc, Mutex},
    thread,
};

use config::Config;
use custom_widgets::{Button, SelectableLabel};
use eframe::{
    egui::{self, RichText, Spinner},
    App,
};
use egui::{CursorIcon::PointingHand as Clickable, Sense};
use hacks::Hack;
use is_elevated::is_elevated;

pub(crate) fn load_icon() -> egui::IconData {
    let (icon_rgba, icon_width, icon_height) = {
        let icon = include_bytes!("../resources/img/icon.ico");
        let image = image::load_from_memory(icon)
            .expect("Failed to open icon path")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };

    egui::IconData {
        rgba: icon_rgba,
        width: icon_width,
        height: icon_height,
    }
}

fn main() {
    let app = MyApp::new();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_min_inner_size(egui::vec2(600.0, 200.0))
            .with_inner_size(egui::vec2(800.0, 400.0))
            .with_icon(std::sync::Arc::new(load_icon())),
        ..Default::default()
    };
    eframe::run_native(
        "AnarchyLoader",
        native_options,
        Box::new(|_cc| Ok(Box::new(app))),
    )
    .unwrap();
}

#[derive(Debug, Clone, PartialEq)]
enum AppTab {
    Home,
    Settings,
    About,
}

impl Default for AppTab {
    fn default() -> Self {
        AppTab::Home
    }
}

struct MyApp {
    items: Vec<Hack>,
    selected_item: Option<Hack>,
    status_message: Arc<Mutex<String>>,
    parse_error: Option<String>,
    app_version: String,
    inject_in_progress: Arc<std::sync::atomic::AtomicBool>,
    tab: AppTab,
    search_query: String,
    config: Config,
}

impl MyApp {
    fn new() -> Self {
        let config = Config::load_config();
        let status_message = Arc::new(Mutex::new(String::new()));
        let inject_in_progress = Arc::new(std::sync::atomic::AtomicBool::new(false));

        let items = match hacks::Hack::fetch_hacks(&config.api_endpoint) {
            Ok(hacks) => hacks,
            Err(err) => {
                return Self {
                    items: Vec::new(),
                    selected_item: None,
                    status_message,
                    parse_error: Some(err),
                    app_version: env!("CARGO_PKG_VERSION").to_string(),
                    inject_in_progress,
                    tab: AppTab::default(),
                    search_query: String::new(),
                    config: config,
                }
            }
        };

        Self {
            items,
            selected_item: None,
            status_message,
            parse_error: None,
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            inject_in_progress,
            tab: AppTab::default(),
            search_query: String::new(),
            config: config,
        }
    }

    fn render_top_panel(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.add_space(5.0);
            ui.horizontal(|ui| {
                if ui
                    .cselectable_label(self.tab == AppTab::Home, "Home")
                    .clicked()
                {
                    self.tab = AppTab::Home;
                }
                if ui
                    .cselectable_label(self.tab == AppTab::Settings, "Settings")
                    .clicked()
                {
                    self.tab = AppTab::Settings;
                }
                if ui
                    .cselectable_label(self.tab == AppTab::About, "About")
                    .clicked()
                {
                    self.tab = AppTab::About;
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

    fn reset_config(&mut self) {
        self.config = Config::default();
        self.config.save_config();
    }

    fn render_home_tab(&mut self, ctx: &egui::Context, theme_color: egui::Color32) {
        let mut items_by_game: BTreeMap<String, BTreeMap<String, Vec<Hack>>> = BTreeMap::new();

        for item in self.items.clone() {
            if self.config.show_only_favorites && !self.config.favorites.contains(&item.name) {
                continue;
            }

            if self.search_query.is_empty()
                || item
                    .name
                    .to_lowercase()
                    .contains(&self.search_query.to_lowercase())
            {
                let game = item.game.clone();
                if game.starts_with("CSS") {
                    let mut parts = game.split_whitespace();
                    let game_name = parts.next().unwrap_or("CSS").to_string();
                    let version = parts.collect::<Vec<&str>>().join(" ");
                    let version = if version.is_empty() {
                        "Unknown version".to_string()
                    } else {
                        version
                    };
                    items_by_game
                        .entry(game_name)
                        .or_insert_with(BTreeMap::new)
                        .entry(version)
                        .or_insert_with(Vec::new)
                        .push(item);
                } else {
                    items_by_game
                        .entry(game.clone())
                        .or_insert_with(BTreeMap::new)
                        .entry("".to_string())
                        .or_insert_with(Vec::new)
                        .push(item);
                }
            }
        }

        items_by_game.retain(|_, versions| {
            versions.retain(|_, items| !items.is_empty());
            !versions.is_empty()
        });

        egui::SidePanel::left("left_panel")
            .resizable(true)
            .default_width(200.0)
            .max_width(300.0)
            .show(ctx, |ui| {
                ui.add_space(5.0);

                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (game_name, versions) in items_by_game {
                        ui.group(|ui| {
                            ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                                ui.with_layout(
                                    egui::Layout::top_down_justified(egui::Align::Center),
                                    |ui| ui.heading(game_name),
                                );
                                ui.separator();
                                for (version, items) in versions {
                                    if !version.is_empty() {
                                        ui.with_layout(
                                            egui::Layout::top_down_justified(egui::Align::Center),
                                            |ui| ui.label(RichText::new(version).heading()),
                                        );
                                    }

                                    for item in items {
                                        let item_clone = item.clone();
                                        ui.horizontal(|ui| {
                                            let is_favorite =
                                                self.config.favorites.contains(&item.name);
                                            let label = if is_favorite {
                                                RichText::new(&item.name)
                                                    .color(self.config.favorites_color)
                                            } else {
                                                RichText::new(&item.name)
                                            };

                                            let response = ui.selectable_label(
                                                self.selected_item.as_ref() == Some(&item),
                                                label,
                                            );

                                            let hovered = response.hovered();

                                            if hovered || is_favorite {
                                                let favorite_icon =
                                                    if is_favorite { "★" } else { "☆" };
                                                if ui
                                                    .add(
                                                        egui::Button::new(RichText::new(
                                                            favorite_icon,
                                                        ))
                                                        .frame(false)
                                                        .sense(Sense::click()),
                                                    )
                                                    .on_hover_cursor(Clickable)
                                                    .clicked()
                                                {
                                                    if is_favorite {
                                                        self.config.favorites.remove(&item.name);
                                                    } else {
                                                        self.config
                                                            .favorites
                                                            .insert(item.name.clone());
                                                    }
                                                    self.config.save_config();
                                                }
                                            }

                                            let file_path_owned = item.file_path.clone();
                                            let ctx_clone = ctx.clone();
                                            let status_message = Arc::clone(&self.status_message);

                                            if response.clicked() {
                                                self.selected_item = Some(item_clone.clone());
                                                let mut status =
                                                    self.status_message.lock().unwrap();
                                                *status = String::new();
                                            }

                                            response.context_menu(|ui| {
                                                if is_favorite {
                                                    if ui.cbutton("Remove from favorites").clicked()
                                                    {
                                                        self.config.favorites.remove(&item.name);
                                                        self.config.save_config();
                                                        ui.close_menu();
                                                    }
                                                } else {
                                                    if ui.cbutton("Add to favorites").clicked()
                                                    {
                                                        self.config.favorites.insert(item.name.clone());
                                                        self.config.save_config();
                                                        ui.close_menu();
                                                    }
                                                }

                                                if Path::new(&file_path_owned).exists() {
                                                    if ui.button_with_tooltip("Open in Explorer", "Open the file location in Explorer").clicked() {
                                                        if let Err(e) = Command::new("explorer.exe")
                                                            .arg(format!(
                                                                "/select,{}",
                                                                item.file_path.to_string_lossy()
                                                            ))
                                                            .spawn()
                                                        {
                                                            let mut status =
                                                                self.status_message.lock().unwrap();
                                                            *status = format!(
                                                                "Failed to open Explorer: {}",
                                                                e
                                                            );
                                                        }
                                                    }
                                                }

                                                if ui.button_with_tooltip("Uninstall", "Uninstall the selected item").clicked() {
                                                        if let Err(e) = std::fs::remove_file(&file_path_owned) {
                                                            let mut status =
                                                                self.status_message.lock().unwrap();
                                                            *status = format!(
                                                                "Failed to uninstall: {}",
                                                                e
                                                            );
                                                        } else {
                                                            let mut status =
                                                                self.status_message.lock().unwrap();
                                                            *status = "Uninstall successful.".to_string();
                                                        }
                                                    }

                                                if ui.button_with_tooltip("Reinstall", "Reinstall the selected item").clicked()
                                                {
                                                    thread::spawn(move || {
                                                        if !Path::new(&file_path_owned).exists() {
                                                            let mut status =
                                                                status_message.lock().unwrap();
                                                            *status =
                                                                "Failed to reinstall: file does not exist.".to_string();
                                                            ctx_clone.request_repaint();
                                                            return;
                                                        }

                                                        {
                                                            let mut status =
                                                                status_message.lock().unwrap();
                                                            *status = "Reinstalling...".to_string();
                                                            ctx_clone.request_repaint();
                                                        }

                                                        if let Err(e) =
                                                            fs::remove_file(&file_path_owned)
                                                        {
                                                            let mut status =
                                                                status_message.lock().unwrap();
                                                            *status = format!(
                                                                "Failed to delete file: {}",
                                                                e
                                                            );
                                                            ctx_clone.request_repaint();
                                                            return;
                                                        }

                                                        match item.download(
                                                            file_path_owned
                                                                .to_string_lossy()
                                                                .to_string(),
                                                        ) {
                                                            Ok(_) => {
                                                                let mut status =
                                                                    status_message.lock().unwrap();
                                                                *status =
                                                                    "Reinstalled.".to_string();
                                                                ctx_clone.request_repaint();
                                                            }
                                                            Err(e) => {
                                                                let mut status =
                                                                    status_message.lock().unwrap();
                                                                *status = format!(
                                                                    "Failed to reinstall: {}",
                                                                    e
                                                                );
                                                                ctx_clone.request_repaint();
                                                            }
                                                        }
                                                    });
                                                }
                                            });
                                            
                                            response.on_hover_cursor(Clickable);
                                        });
                                    }
                                }
                            });
                        });
                        ui.add_space(10.0);
                    }
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(10.0);
            if let Some(selected) = &self.selected_item {
                ui.horizontal(|ui| {
                    ui.heading(&selected.name);
                    ui.label(RichText::new(format!("by {}", selected.author)).color(theme_color));
                    ui.hyperlink_to("(source)", &selected.source)
                        .on_hover_text(&selected.source)
                });
                ui.separator();
                ui.label(&selected.description);

                let is_csgo = selected.game == "CSGO";

                if ui.button(format!("Inject {}", selected.name))
                    .on_hover_cursor(Clickable)
                    .on_hover_text(&selected.file)
                    .clicked()
                {
                    if is_csgo {
                        self.manual_map_injection(selected.clone(), ctx.clone());
                    } else {
                        self.start_injection(selected.clone(), ctx.clone());
                    }
                }
                
                if !is_elevated() && is_csgo && !self.config.hide_csgo_warning {
                    ui.label(
                        RichText::new("If you encounter an error stating that csgo.exe is not found\ntry running the loader as an administrator.")
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
            } else {
                ui.vertical_centered(|ui| {
                    ui.add_space(150.0);
                    ui.label("Please select a cheat from the list.");
                });
            }
        });
    }

    fn render_settings_tab(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
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
                .checkbox(
                    &mut self.config.hide_csgo_warning,
                    "Hide CSGO warning"
                )
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

            ui.add_space(10.0);

            if ui.cbutton("Reset settings").clicked() {
                self.reset_config();
            }
        });
    }

    fn render_about_tab(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("About");
            ui.separator();
            ui.label(RichText::new(format!("AnarchyLoader v{}", self.app_version)).size(24.0));
            ui.add(
                egui::Image::new(egui::include_image!("../resources/img/icon.ico"))
                    .max_width(100.0)
                    .rounding(10.0),
            );
            ui.add_space(10.0);
            ui.label("AnarchyLoader is a free and open-source cheat loader for various games.");
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                if ui.cbutton("Visit Website").clicked() {
                    let _ = opener::open("https://anarchy.my");
                }
                if ui.cbutton("Github Repository").clicked() {
                    let _ = opener::open("https://github.com/AnarchyLoader/AnarchyLoader");
                }
            });
        });
    }
}

impl App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui_extras::install_image_loaders(ctx);

        let is_dark_mode = ctx.style().visuals.dark_mode;
        let theme_color = if is_dark_mode {
            egui::Color32::LIGHT_GRAY
        } else {
            egui::Color32::DARK_GRAY
        };

        if self.parse_error.is_some() {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(130.0);
                    ui.colored_label(
                        egui::Color32::RED,
                        RichText::new(self.parse_error.as_ref().unwrap())
                            .size(24.0)
                            .strong(),
                    );

                    ui.label("API Endpoint (editable):");
                    if ui
                        .text_edit_singleline(&mut self.config.api_endpoint)
                        .changed()
                    {
                        self.config.save_config();
                    }
                });
            });
            return;
        }

        self.render_top_panel(ctx);

        match self.tab {
            AppTab::Home => self.render_home_tab(ctx, theme_color),
            AppTab::Settings => self.render_settings_tab(ctx),
            AppTab::About => self.render_about_tab(ctx),
        }
    }
}
