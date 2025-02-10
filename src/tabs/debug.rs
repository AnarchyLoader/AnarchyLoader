use egui::RichText;

use crate::{utils::ui::custom_widgets::Button, MyApp};

impl MyApp {
    pub fn render_debug_tab(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .drag_to_scroll(false)
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());

                    ui.collapsing("Variables", |ui| {
                        let debug_info = vec![
                            ("Hacks:", format!("{:#?}", self.app.hacks)),
                            ("Config:", format!("{:#?}", self.app.config)),
                            ("Statistics:", format!("{:#?}", self.app.stats)),
                            ("Updater:", format!("{:#?}", self.app.updater)),
                            ("Cache:", format!("{:#?}", self.app.cache)),
                            ("Ui states:", format!("{:#?}", self.ui)),
                            ("Communication:", format!("{:#?}", self.communication)),
                        ];

                        for (label, value) in &debug_info {
                            if label.starts_with("Hacks") {
                                ui.collapsing(*label, |ui| {
                                    for hack in &self.app.hacks {
                                        ui.monospace(format!("{:#?}", hack));
                                    }
                                });
                                continue;
                            } else {
                                ui.separator();
                                ui.label(RichText::new(*label).size(12.5));
                                ui.separator();
                                ui.monospace(value);
                            }

                            ui.add_space(10.0);
                        }

                        if ui.cbutton("Copy debug info").clicked() {
                            let debug_info = "```\n".to_string()
                                + &debug_info
                                    .iter()
                                    .filter(|(label, _)| !label.starts_with("Hacks"))
                                    .map(|(label, value)| format!("{} {}\n", label, value))
                                    .collect::<String>()
                                + "```";
                            ui.output_mut(|o| o.copied_text = debug_info);
                            self.toasts.success("Debug info copied to clipboard.");
                        }
                    });
                });
        });
    }
}
