use egui_alignments::center_vertical;
use egui_material_icons::icons::ICON_ENGINEERING;

use crate::MyApp;

impl MyApp {
    pub fn render_cfgs_tab(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            center_vertical(ui, |ui| {
                ui.heading(format!("{} Page under construction.", ICON_ENGINEERING));
            });
        });
    }
}
