use crate::{custom_widgets::Button, MyApp};

impl MyApp {
    pub fn render_debug_tab(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .drag_to_scroll(false)
                .show(ui, |ui| {
                    ui.heading("Debug");
                    ui.separator();

                    let debug_info = vec![
                        ("Config:", format!("{:#?}", self.config)),
                        ("Hacks:", format!("{:#?}", self.hacks)),
                        ("Selected Hack:", format!("{:#?}", self.selected_hack)),
                        ("Status Message:", format!("{:#?}", self.status_message)),
                        ("Parse Error:", format!("{:#?}", self.parse_error)),
                        (
                            "Inject in Progress:",
                            format!("{:#?}", self.inject_in_progress),
                        ),
                        ("Search Query:", format!("{:#?}", self.search_query)),
                        (
                            "Main Menu Message:",
                            format!("{:#?}", self.main_menu_message),
                        ),
                        ("App Version:", format!("{:#?}", self.app_version)),
                    ];

                    for (label, value) in &debug_info {
                        if label.starts_with("Hacks") {
                            ui.collapsing(*label, |ui| {
                                for hack in &self.hacks {
                                    ui.monospace(format!("{:#?}", hack));
                                }
                            });
                            continue;
                        } else {
                            ui.label(*label);
                            ui.monospace(value);
                        }

                        ui.add_space(10.0);
                    }

                    if ui.cbutton("Copy debug info").clicked() {
                        let debug_info = debug_info
                            .iter()
                            .filter(|(label, _)| !label.starts_with("Hacks"))
                            .map(|(label, value)| format!("{}: {}\n", label, value))
                            .collect::<String>();
                        ui.output_mut(|o| o.copied_text = debug_info);
                        self.toasts.success("Debug info copied to clipboard.");
                    }
                });
        });
    }
}
