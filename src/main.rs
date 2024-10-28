#![windows_subsystem = "windows"]

mod downloader;
mod hacks;

use eframe::{
    egui::{self, RichText, Spinner},
    App,
};

use dll_syringe::{process::OwnedProcess, Syringe};
use hacks::Hack;
use std::{
    env,
    time::{Duration, Instant},
};

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
    let app = MyApp::default();

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

struct MyApp {
    items: Vec<Hack>,
    selected_item: Option<Hack>,
    status_message: String,
    app_version: String,
    inject_in_progress: bool,
    injecting_start_time: Option<Instant>,
}

impl Default for MyApp {
    fn default() -> Self {
        let items = vec![
            Hack::new("HPP v6", "HVH Cheat", "_xvi", "crack", "hpp_v6", ""),
            Hack::new(
                "Sakura",
                "Sakura is a free and public cheat for Counter-Strike 1.6 written in C++.",
                "nc-gp",
                "open-source",
                "sakura",
                "",
            ),
            Hack::new(
                "Dopamine",
                "CS 1.6 Multihack. Attempt to develop and improve Nor-Adrenaline.",
                "KleskBY",
                "open-source",
                "dopamine",
                "",
            ),
            Hack::new(
                "AimWare",
                "AimWare is cheat mod for CS 1.6, inspired by the Aimware CS",
                "rushensky",
                "crack",
                "AimWare",
                ""
            )

        ];

        Self {
            items: items.clone(),
            selected_item: None,
            status_message: String::new(),
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            inject_in_progress: false,
            injecting_start_time: None,
        }
    }
}

impl App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::left("left_panel").show(ctx, |ui| {
            ui.add_space(5.0);

            ui.with_layout(
                egui::Layout::top_down_justified(egui::Align::Center),
                |ui| {
                    for item in &self.items {
                        if ui
                            .selectable_label(
                                self.selected_item.as_ref() == Some(item),
                                item.name.clone(),
                            )
                            .clicked()
                        {
                            self.selected_item = Some(item.clone());
                            self.status_message.clear();
                        }
                    }
                },
            );
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.centered_and_justified(|ui| {
                ui.vertical_centered(|ui| {
                    ui.label(
                        RichText::new(format!("AnarchyLoader v{}", self.app_version)).size(24.0),
                    );
                    ui.separator();

                    if let Some(selected) = &self.selected_item {
                        ui.add_space(130.0);
                        ui.vertical_centered(|ui| {
                            ui.label(
                                RichText::new(format!("{} by {}", selected.name, selected.author))
                                    .size(24.0),
                            );
                            ui.label(RichText::new(format!("{}", selected.status))
                                .color(egui::Color32::LIGHT_BLUE));

                            ui.label(RichText::new(selected.description.clone()).heading());
                        });

                        ui.add_space(3.0);

                        if ui
                            .button(format!("Inject the {}", selected.name))
                            .on_hover_cursor(egui::CursorIcon::PointingHand)
                            .on_hover_text(format!("Inject the {}", selected.name))
                            .clicked()
                        {
                            self.status_message = "Injecting...".to_string();
                            self.inject_in_progress = true;
                            self.injecting_start_time = Some(Instant::now());
                            ctx.request_repaint();
                        }

                        if self.inject_in_progress {
                            if let Some(start_time) = self.injecting_start_time {
                                if start_time.elapsed() >= Duration::from_secs(2) {
                                    let temp_dir = env::temp_dir();
                                    let file_path =
                                        format!("{}{}.dll", temp_dir.display(), selected.name);

                                    selected.download(&mut self.status_message, file_path.clone());
                                    if let Some(target_process) =
                                        OwnedProcess::find_first_by_name("hl.exe")
                                    {
                                        let syringe = Syringe::for_process(target_process);
                                        if let Err(e) = syringe.inject(file_path) {
                                            self.status_message =
                                                format!("Failed to inject: {}", e);
                                        } else {
                                            self.status_message =
                                                "Injection successful.".to_string();
                                        }
                                    } else {
                                        self.status_message =
                                            "Failed to inject: Process 'hl.exe' not found."
                                                .to_string();
                                    }
                                    self.inject_in_progress = false;
                                    self.injecting_start_time = None;
                                } else {
                                    ui.label(RichText::new(&self.status_message).color(
                                        if self.status_message.starts_with("Failed") {
                                            egui::Color32::RED
                                        } else {
                                            egui::Color32::WHITE
                                        },
                                    ));
                                    ui.add(Spinner::default());

                                    ctx.request_repaint();
                                }
                            }
                        } else if !self.status_message.is_empty() {
                            let color = if self.status_message.starts_with("Failed") {
                                egui::Color32::RED
                            } else {
                                egui::Color32::WHITE
                            };
                            ui.label(RichText::new(&self.status_message).color(color));
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
