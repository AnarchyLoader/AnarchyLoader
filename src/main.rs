#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod downloader;
mod hacks;

use std::collections::BTreeMap;
use std::env;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use dll_syringe::{process::OwnedProcess, Syringe};
use eframe::{
    egui::{self, RichText, Spinner},
    App,
};
use reqwest::blocking::Client;
use serde::Deserialize;

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
            .with_resizable(false)
            .with_maximize_button(false)
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

struct MyApp {
    items: Vec<Hack>,
    selected_item: Option<Hack>,
    status_message: Arc<Mutex<String>>,
    parse_error: Option<String>,
    app_version: String,
    inject_in_progress: Arc<std::sync::atomic::AtomicBool>,
    tab: AppTab,
    search_query: String,
}

impl MyApp {
    fn new() -> Self {
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
                    .clicked()
                {
                    self.tab = AppTab::Home;
                }
                if ui
                    .selectable_label(self.tab == AppTab::Settings, "Settings")
                    .clicked()
                {
                    self.tab = AppTab::Settings;
                }
                if ui
                    .selectable_label(self.tab == AppTab::About, "About")
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

    fn render_home_tab(&mut self, ctx: &egui::Context, theme_color: egui::Color32) {
        let mut items_by_game: BTreeMap<String, BTreeMap<String, Vec<&Hack>>> = BTreeMap::new();

        for item in &self.items {
            if self.search_query.is_empty()
                || item
                    .name
                    .to_lowercase()
                    .contains(&self.search_query.to_lowercase())
            {
                let game = item.game.clone();
                if game.starts_with("CSS") {
                    let mut parts = game.split_whitespace();
                    let game_name = parts.next().unwrap_or("CSS").to_string(); // "CSS"
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
                        .entry("".to_string()) // No version
                        .or_insert_with(Vec::new)
                        .push(item);
                }
            }
        }

        egui::SidePanel::left("left_panel")
            .resizable(true)
            .default_width(200.0)
            .show(ctx, |ui| {
                ui.add_space(5.0);

                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (game_name, versions) in &items_by_game {
                        ui.group(|ui| {
                            ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                                ui.heading(game_name);
                                ui.separator();
                                for (version, items) in versions {
                                    if !version.is_empty() {
                                        ui.label(format!("Version: {}", version));
                                    }
                                    for item in items {
                                        let is_selected =
                                            self.selected_item.as_ref() == Some(*item);
                                        if ui
                                            .selectable_label(is_selected, &item.name)
                                            .on_hover_cursor(egui::CursorIcon::PointingHand)
                                            .clicked()
                                        {
                                            self.selected_item = Some((*item).clone());
                                            self.status_message.lock().unwrap().clear();
                                        }
                                    }
                                    ui.add_space(5.0);
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
                    .on_hover_cursor(egui::CursorIcon::PointingHand)
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

                        let temp_dir = env::temp_dir();
                        let file_path = temp_dir.join(&selected_clone.file);

                        if !file_path.exists() {
                            {
                                let mut status = status_message.lock().unwrap();
                                *status = "Downloading...".to_string();
                            }
                            ctx_clone.request_repaint();

                            match selected_clone.download(file_path.to_string_lossy().to_string()) {
                                Ok(_) => {
                                    {
                                        let mut status = status_message.lock().unwrap();
                                        *status = "Download complete.".to_string();
                                    }
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
                            if let Err(e) = syringe.inject(file_path) {
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
            ui.label("Nothing to see here yet.");
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
                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                    .clicked()
                {
                    let _ = opener::open("https://anarchy.collapseloader.org");
                }
                if ui
                    .button("Github Repository")
                    .on_hover_cursor(egui::CursorIcon::PointingHand)
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
