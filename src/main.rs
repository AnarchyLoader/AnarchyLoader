#![windows_subsystem = "windows"]

mod downloader;

use eframe::{
    egui::{self, RichText},
    App,
};

use dll_syringe::{process::OwnedProcess, Syringe};

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
    items: Vec<String>,
    selected_item: String,
    error_message: String,
}

impl Default for MyApp {
    fn default() -> Self {
        let items = vec![
            "HPP v6".to_string(),
            "Sakura".to_string(),
            "Dopamine".to_string(),
        ];

        Self {
            items,
            selected_item: String::new(),
            error_message: String::new(),
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
                            .selectable_label(self.selected_item == *item, item)
                            .clicked()
                        {
                            self.selected_item = item.clone();
                            self.error_message.clear();
                        }
                    }
                },
            );
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.centered_and_justified(|ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("AnarchyLoader");
                    ui.separator();

                    if !self.selected_item.is_empty() {
                        ui.add_space(100.0);
                        ui.label(RichText::new(self.selected_item.clone()).size(24.0));
                        ui.add_space(10.0);

                        if ui
                            .button("Inject")
                            .on_hover_cursor(egui::CursorIcon::PointingHand)
                            .on_hover_text(format!("Inject the {}", self.selected_item))
                            .clicked()
                        {
                            let file_path = format!("{}.dll", self.selected_item);
                            if !std::path::Path::new(&file_path).exists() {
                                match downloader::download_file(&self.selected_item, &file_path) {
                                    Ok(_) => (),
                                    Err(e) => {
                                        self.error_message =
                                            format!("Failed to download file: {}", e);
                                        return;
                                    }
                                }
                            }

                            if let Some(target_process) = OwnedProcess::find_first_by_name("hl.exe")
                            {
                                let syringe = Syringe::for_process(target_process);
                                if let Err(e) =
                                    syringe.inject(format!("{}.dll", self.selected_item))
                                {
                                    self.error_message = format!("Failed to inject: {}", e);
                                } else {
                                    self.error_message.clear();
                                }
                            } else {
                                self.error_message = "Process 'hl.exe' not found.".to_string();
                            }
                        }

                        if !self.error_message.is_empty() {
                            ui.label(RichText::new(&self.error_message).color(egui::Color32::RED));
                        }
                    } else {
                        ui.label("Please select an item from the list.");
                    }
                });
            });
        });
    }
}
