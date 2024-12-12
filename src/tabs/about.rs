use egui::{RichText, Sense};

use crate::{custom_widgets::{Button, Hyperlink}, MyApp};

impl MyApp {
    pub fn render_about_tab(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading("About");
                ui.separator();
                if ui
                    .add(
                        egui::Image::new(egui::include_image!("../../resources/img/icon.ico"))
                            .max_width(100.0)
                            .rounding(10.0)
                            .sense(Sense::click()),
                    )
                    .clicked()
                {
                    self.toasts.info("Hello there!");
                }
                ui.label(RichText::new(format!("v{}", self.app_version)).size(15.0));
                ui.add_space(10.0);
                ui.label(
                    RichText::new(
                        "AnarchyLoader is a free and open-source cheat loader for various games.",
                    )
                    .size(16.0),
                );
                ui.add_space(5.0);
                ui.horizontal(|ui| {
                    ui.clink("Made with egui", "https://www.github.com/emilk/egui/");
                    ui.clink("by dest4590", "https://github.com/dest4590");
                });
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if ui.cbutton("Visit Website").clicked() {
                        let _ = opener::open("https://anarchy.my");
                    }
                    if ui.cbutton("Github Repository").clicked() {
                        let _ = opener::open("https://github.com/AnarchyLoader/AnarchyLoader");
                    }
                });

                ui.add_space(5.0);
                ui.label("Keybinds:");
                ui.label("F5 - Refresh hacks");
                ui.label("Enter - Inject selected hack");
                ui.label("Escape - Deselect hack");
                ui.label("Hold Alt - Debug tab");
            });
        });
        self.toasts.show(ctx);
    }
}