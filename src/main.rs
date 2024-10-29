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

#[derive(Deserialize, Debug)]
struct HackApiResponse {
    name: String,
    description: String,
    author: String,
    status: String,
    file: String,
    process: String,
}

struct MyApp {
    items: Vec<Hack>,
    games: BTreeMap<String, String>,
    selected_item: Option<Hack>,
    status_message: Arc<Mutex<String>>,
    parse_error: Option<String>,
    app_version: String,
    inject_in_progress: Arc<Mutex<bool>>,
}

impl MyApp {
    fn new() -> Self {
        let mut items = Vec::new();
        let mut parse_error = None;
        let status_message = Arc::new(Mutex::new(String::new()));
        let inject_in_progress = Arc::new(Mutex::new(false));

        let client = Client::new();
        let api_url = if std::env::args().any(|arg| arg == "--local") {
            "http://127.0.0.1:8000/api/hacks/"
        } else {
            "https://anarchy.collapseloader.org/api/hacks/"
        };

        let response = client.get(api_url).send();

        match response {
            Ok(res) => {
                if res.status().is_success() {
                    match res.json::<Vec<HackApiResponse>>() {
                        Ok(parsed_hacks) => {
                            if parsed_hacks.is_empty() {
                                parse_error = Some("No hacks available.".to_string());
                            } else {
                                for hack in parsed_hacks {
                                    items.push(Hack::new(
                                        &hack.name,
                                        &hack.description,
                                        &hack.author,
                                        &hack.status,
                                        &hack.file,
                                        &hack.process,
                                    ));
                                }
                            }
                        }
                        Err(err) => {
                            parse_error = Some(format!("Failed to parse JSON: {}", err));
                        }
                    }
                } else {
                    parse_error = Some(format!("API request failed with status: {}", res.status()));
                }
            }
            Err(err) => parse_error = Some(format!("API request failed: {}", err)),
        }

        let mut games = BTreeMap::new();
        games.insert("hl.exe".to_string(), "CS 1.6".to_string());

        Self {
            items,
            games,
            selected_item: None,
            status_message,
            parse_error,
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            inject_in_progress,
        }
    }
}

impl App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(error) = &self.parse_error {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(130.0);
                    ui.colored_label(egui::Color32::RED, RichText::new(error).size(24.0).strong());
                });
            });
            return;
        }

        let mut items_by_process: BTreeMap<String, Vec<&Hack>> = BTreeMap::new();

        for item in &self.items {
            items_by_process
                .entry(item.process.clone())
                .or_insert_with(Vec::new)
                .push(item);
        }

        egui::SidePanel::left("left_panel").show(ctx, |ui| {
            ui.add_space(5.0);

            for (process, items) in &items_by_process {
                ui.group(|ui| {
                    ui.with_layout(
                        egui::Layout::top_down_justified(egui::Align::Center),
                        |ui| {
                            ui.label(format!("{}", self.games.get(process).unwrap()));
                            ui.separator();
                            for item in items {
                                if ui
                                    .selectable_label(
                                        self.selected_item.as_ref() == Some(&(*item).clone()),
                                        item.name.clone(),
                                    )
                                    .clicked()
                                {
                                    self.selected_item = Some((*item).clone());
                                    let mut status = self.status_message.lock().unwrap();
                                    status.clear();
                                }
                            }
                        },
                    );
                });
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.centered_and_justified(|ui| {
                ui.vertical_centered(|ui| {
                    ui.label(
                        RichText::new(format!("AnarchyLoader v{}", self.app_version)).size(24.0),
                    );
                    ui.separator();

                    if let Some(selected) = &self.selected_item {
                        ui.add_space(100.0);
                        ui.vertical_centered(|ui| {
                            ui.label(RichText::new(selected.name.clone()).size(24.0));
                            ui.label(
                                RichText::new(format!(
                                    "{} | by {}",
                                    selected.status, selected.author
                                ))
                                .color(egui::Color32::LIGHT_BLUE),
                            );

                            ui.label(RichText::new(selected.description.clone()).size(14.0));
                        });

                        ui.add_space(3.0);

                        if ui
                            .button(format!("Inject the {}", selected.name))
                            .on_hover_cursor(egui::CursorIcon::PointingHand)
                            .on_hover_text(format!("Inject the {}", selected.name))
                            .clicked()
                        {
                            let inject_in_progress = Arc::clone(&self.inject_in_progress);
                            let status_message = Arc::clone(&self.status_message);
                            let selected_clone = selected.clone();
                            let ctx_clone = ctx.clone();

                            *inject_in_progress.lock().unwrap() = true;

                            thread::spawn(move || {
                                {
                                    let mut status = status_message.lock().unwrap();
                                    *status = "Starting injection...".to_string();
                                }
                                ctx_clone.request_repaint();
                                thread::sleep(Duration::from_secs(1));

                                let temp_dir = env::temp_dir();
                                let file_path =
                                    format!("{}{}.dll", temp_dir.display(), selected_clone.file);

                                if !std::path::Path::new(&file_path).exists() {
                                    {
                                        let mut status = status_message.lock().unwrap();
                                        *status = "Downloading...".to_string();
                                    }
                                    ctx_clone.request_repaint();

                                    let download_result =
                                        selected_clone.download(file_path.clone());

                                    match download_result {
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
                                            ctx_clone.request_repaint();
                                            *inject_in_progress.lock().unwrap() = false;
                                            return;
                                        }
                                    }
                                } else {
                                    {
                                        let mut status = status_message.lock().unwrap();
                                        *status =
                                            "File already exists. Skipping download.".to_string();
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
                                    OwnedProcess::find_first_by_name("hl.exe")
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
                                    *status =
                                        "Failed to inject: Process 'hl.exe' not found.".to_string();
                                }

                                *inject_in_progress.lock().unwrap() = false;
                                ctx_clone.request_repaint();
                            });
                        }

                        let inject_in_progress = *self.inject_in_progress.lock().unwrap();

                        if inject_in_progress {
                            ui.add_space(10.0);
                            let status = self.status_message.lock().unwrap().clone();
                            ui.label(RichText::new(&status).color(
                                if status.starts_with("Failed") {
                                    egui::Color32::RED
                                } else {
                                    egui::Color32::WHITE
                                },
                            ));
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
                                    egui::Color32::WHITE
                                };
                                ui.label(RichText::new(&status).color(color));
                            }
                        }
                    } else {
                        ui.add_space(150.0);
                        ui.label("Please select a cheat from the list.");
                    }
                });
            });
        });
    }
}
