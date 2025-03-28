use egui::RichText;

use crate::{utils::ui::widgets::Button, MyApp};

impl MyApp {
    pub fn render_debug_tab(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .drag_to_scroll(false)
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());

                    let debug_info = vec![
                        ("Hacks:", format!("{:#?}", self.app.hacks)),
                        ("Config:", format!("{:#?}", self.app.config)),
                        ("Statistics:", format!("{:#?}", self.app.stats)),
                        ("Updater:", format!("{:#?}", self.app.updater)),
                        ("Cache:", format!("{:#?}", self.ui.mark_cache)),
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
                        #[allow(clippy::format_collect)]
                        let debug_info = "```\n".to_string()
                            + &debug_info
                            .iter()
                            .filter(|(label, _)| !label.starts_with("Hacks"))
                            .map(|(label, value)| format!("{} {}\n", label, value))
                            .collect::<String>()
                            + "```";
                        ctx.copy_text(debug_info.clone());
                        self.toasts.success("Debug info copied to clipboard.");
                    }
                });
        });
    }
}
