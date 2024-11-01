#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod downloader;
mod hacks;

use std::collections::{BTreeMap, HashSet};
use std::path::Path;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use std::{env, fs};

use dll_syringe::{process::OwnedProcess, Syringe};
use eframe::{
    egui::{self, RichText, Spinner},
    App,
};
use egui::CursorIcon::PointingHand as Clickable;
use egui::Sense;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

use hacks::Hack;

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

#[derive(Deserialize, Debug, Clone, PartialEq)]
struct HackApiResponse {
    name: String,
    description: String,
    author: String,
    status: String,
    file: String,
    process: String,
    source: String,
    game: String,
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

#[derive(Serialize, Deserialize)]
struct Config {
    favorites: HashSet<String>,
    show_only_favorites: bool,
    favorites_color: egui::Color32,
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
    favorites: HashSet<String>,
    show_only_favorites: bool,
    favorites_color: egui::Color32,
}

impl MyApp {
    fn new() -> Self {
        let config = Self::load_favorites();
        let status_message = Arc::new(Mutex::new(String::new()));
        let inject_in_progress = Arc::new(std::sync::atomic::AtomicBool::new(false));

        let items = match Self::fetch_hacks() {
            Ok(hacks) => hacks,
            Err(err) => {
                return Self {
                    parse_error: Some(err),
                    items: Vec::new(),
                    selected_item: None,
                    status_message,
                    app_version: env!("CARGO_PKG_VERSION").to_string(),
                    inject_in_progress,
                    tab: AppTab::default(),
                    search_query: String::new(),
                    favorites: config.favorites,
                    show_only_favorites: config.show_only_favorites,
                    favorites_color: config.favorites_color,
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
            favorites: config.favorites,
            show_only_favorites: config.show_only_favorites,
            favorites_color: config.favorites_color,
        }
    }

    fn fetch_hacks() -> Result<Vec<Hack>, String> {
        let client = Client::new();
        let api_url = if std::env::args().any(|arg| arg == "--local") {
            "http://127.0.0.1:8000/api/hacks/"
        } else {
            "https://anarchy.collapseloader.org/api/hacks/"
        };

        let res = client.get(api_url).send().map_err(|e| e.to_string())?;

        if res.status().is_success() {
            let parsed_hacks: Vec<HackApiResponse> = res.json().map_err(|e| e.to_string())?;
            if parsed_hacks.is_empty() {
                Err("No hacks available.".to_string())
            } else {
                Ok(parsed_hacks
                    .into_iter()
                    .map(|hack| {
                        Hack::new(
                            &hack.name,
                            &hack.description,
                            &hack.author,
                            &hack.status,
                            &hack.file,
                            &hack.process,
                            &hack.source,
                            &hack.game,
                        )
                    })
                    .collect())
            }
        } else {
            Err(format!("API request failed with status: {}", res.status()))
        }
    }

    fn render_top_panel(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.add_space(5.0);
            ui.horizontal(|ui| {
                if ui
                    .selectable_label(self.tab == AppTab::Home, "Home")
                    .on_hover_cursor(Clickable)
                    .clicked()
                {
                    self.tab = AppTab::Home;
                }
                if ui
                    .selectable_label(self.tab == AppTab::Settings, "Settings")
                    .on_hover_cursor(Clickable)
                    .clicked()
                {
                    self.tab = AppTab::Settings;
                }
                if ui
                    .selectable_label(self.tab == AppTab::About, "About")
                    .on_hover_cursor(Clickable)
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

    fn load_favorites() -> Config {
        let config_path = dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("anarchyloader")
            .join("config.json");

        if let Ok(data) = fs::read_to_string(config_path) {
            if let Ok(config) = serde_json::from_str::<Config>(&data) {
                return config;
            }
        }
        Config {
            favorites: HashSet::new(),
            show_only_favorites: false,
            favorites_color: egui::Color32::GOLD,
        }
    }

    fn save_favorites(&self) {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("anarchyloader");

        fs::create_dir_all(&config_dir).ok();
        let config_path = config_dir.join("config.json");

        let config = Config {
            favorites: self.favorites.clone(),
            show_only_favorites: self.show_only_favorites,
            favorites_color: self.favorites_color,
        };

        if let Ok(data) = serde_json::to_string(&config) {
            fs::write(config_path, data).ok();
        }
    }

    fn render_home_tab(&mut self, ctx: &egui::Context, theme_color: egui::Color32) {
        let mut items_by_game: BTreeMap<String, BTreeMap<String, Vec<Hack>>> = BTreeMap::new();

        for item in self.items.clone() {
            if self.show_only_favorites && !self.favorites.contains(&item.name) {
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
            .show(ctx, |ui| {
                ui.add_space(5.0);

                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (game_name, versions) in items_by_game {
                        ui.group(|ui| {
                            ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                                ui.heading(game_name);
                                ui.separator();
                                for (version, items) in versions {
                                    if !version.is_empty() {
                                        ui.label(RichText::new(version).heading());
                                    }

                                    for item in items {
                                        let item_clone = item.clone();

                                        ui.horizontal(|ui| {
                                            let is_favorite = self.favorites.contains(&item.name);
                                            let label = if is_favorite {
                                                RichText::new(&item.name)
                                                    .color(self.favorites_color)
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
                                                        self.favorites.remove(&item.name);
                                                    } else {
                                                        self.favorites.insert(item.name.clone());
                                                    }
                                                    self.save_favorites();
                                                }
                                            }
                                            let file_path_owned = item.file_path.clone();
                                            let ctx_clone = ctx.clone();
                                            let status_message = Arc::clone(&self.status_message);

                                            response.context_menu(|ui| {
                                                if is_favorite {
                                                    if ui
                                                        .button("Remove from favorites")
                                                        .on_hover_cursor(Clickable)
                                                        .clicked()
                                                    {
                                                        self.favorites.remove(&item.name);
                                                        self.save_favorites();
                                                        ui.close_menu();
                                                    }
                                                } else {
                                                    if ui
                                                        .button("Add to favorites")
                                                        .on_hover_cursor(Clickable)
                                                        .clicked()
                                                    {
                                                        self.favorites.insert(item.name.clone());
                                                        self.save_favorites();
                                                        ui.close_menu();
                                                    }
                                                }

                                                if ui
                                                    .button("Open in Explorer")
                                                    .on_hover_cursor(Clickable)
                                                    .on_hover_text(
                                                        "Open the file location in Explorer",
                                                    )
                                                    .clicked()
                                                {
                                                    if !Path::new(&file_path_owned).exists() {
                                                        let mut status =
                                                            status_message.lock().unwrap();
                                                        *status =
                                                            "Failed: File does not exist.".to_string();
                                                        ctx_clone.request_repaint();
                                                        return;
                                                    }

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
                                                if ui
                                                    .button("Reinstall")
                                                    .on_hover_cursor(Clickable)
                                                    .on_hover_text("Reinstall the selected item")
                                                    .clicked()
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

                                            if response.clicked() {
                                                self.selected_item = Some(item_clone.clone());
                                                let mut status =
                                                    self.status_message.lock().unwrap();
                                                *status = String::new();
                                            }

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
                });
                ui.separator();
                ui.label(&selected.description);

                if ui
                    .button(format!("Inject {}", selected.name))
                    .on_hover_cursor(Clickable)
                    .on_hover_text(format!("Inject the {}", selected.name))
                    .clicked()
                {
                    let inject_in_progress = Arc::clone(&self.inject_in_progress);
                    let status_message = Arc::clone(&self.status_message);
                    let selected_clone = selected.clone();
                    let ctx_clone = ctx.clone();

                    {
                        let mut status = status_message.lock().unwrap();
                        *status = "Starting injection...".to_string();
                    }

                    inject_in_progress.store(true, std::sync::atomic::Ordering::SeqCst);

                    thread::spawn(move || {
                        ctx_clone.request_repaint();
                        thread::sleep(Duration::from_secs(1));

                        if !selected_clone.file_path.exists() {
                            {
                                let mut status = status_message.lock().unwrap();
                                *status = "Downloading...".to_string();
                            }
                            ctx_clone.request_repaint();

                            match selected_clone
                                .download(selected_clone.file_path.to_string_lossy().to_string())
                            {
                                Ok(_) => {
                                    ctx_clone.request_repaint();
                                }
                                Err(e) => {
                                    let mut status = status_message.lock().unwrap();
                                    *status = format!("Failed to download: {}", e);
                                    inject_in_progress
                                        .store(false, std::sync::atomic::Ordering::SeqCst);
                                    ctx_clone.request_repaint();
                                    return;
                                }
                            }
                        } else {
                            {
                                let mut status = status_message.lock().unwrap();
                                *status = "File already exists. Skipping download.".to_string();
                            }
                            ctx_clone.request_repaint();
                        }

                        thread::sleep(Duration::from_secs(1));

                        {
                            let mut status = status_message.lock().unwrap();
                            *status = "Injecting...".to_string();
                        }
                        ctx_clone.request_repaint();
                        thread::sleep(Duration::from_secs(1));

                        if let Some(target_process) =
                            OwnedProcess::find_first_by_name(&selected_clone.process)
                        {
                            let syringe = Syringe::for_process(target_process);
                            if let Err(e) = syringe.inject(selected_clone.file_path.clone()) {
                                let mut status = status_message.lock().unwrap();
                                *status = format!("Failed to inject: {}", e);
                            } else {
                                let mut status = status_message.lock().unwrap();
                                *status = "Injection successful.".to_string();
                            }
                        } else {
                            let mut status = status_message.lock().unwrap();
                            *status = format!(
                                "Failed to inject: Process '{}' not found.",
                                selected_clone.process
                            );
                        }

                        inject_in_progress.store(false, std::sync::atomic::Ordering::SeqCst);
                        ctx_clone.request_repaint();
                    });
                }

                let inject_in_progress = self
                    .inject_in_progress
                    .load(std::sync::atomic::Ordering::SeqCst);

                if inject_in_progress {
                    ui.add_space(10.0);
                    let status = self.status_message.lock().unwrap().clone();
                    ui.label(
                        RichText::new(&status).color(if status.starts_with("Failed") {
                            egui::Color32::RED
                        } else {
                            theme_color
                        }),
                    );
                    ui.add_space(2.0);
                    ui.add(Spinner::new());
                    ctx.request_repaint();
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
                .checkbox(&mut self.show_only_favorites, "Show only favorite hacks")
                .on_hover_cursor(Clickable)
                .changed()
            {
                self.save_favorites();
            }
            ui.add_space(10.0);
            ui.label("Favorites Color:");
            if ui
                .color_edit_button_srgba(&mut self.favorites_color)
                .on_hover_cursor(Clickable)
                .changed()
            {
                self.save_favorites();
            }
        });
    }

    fn render_about_tab(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("About");
            ui.separator();
            ui.label(format!("AnarchyLoader v{}", self.app_version));
            ui.add_space(10.0);
            ui.label("AnarchyLoader is a free and open-source cheat loader for various games.");
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                if ui
                    .button("Visit Website")
                    .on_hover_cursor(Clickable)
                    .clicked()
                {
                    let _ = opener::open("https://anarchy.collapseloader.org");
                }
                if ui
                    .button("Github Repository")
                    .on_hover_cursor(Clickable)
                    .clicked()
                {
                    let _ = opener::open("https://github.com/AnarchyLoader/AnarchyLoader");
                }
            });
        });
    }
}

impl App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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
