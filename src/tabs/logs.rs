use egui::{RichText, TextStyle};
use log::Level;

use crate::MyApp;

impl MyApp {
    pub fn render_logs_tab(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Log Level:");
                for &level in &[
                    Level::Error,
                    Level::Warn,
                    Level::Info,
                    Level::Debug,
                    Level::Trace,
                ] {
                    if ui
                        .radio_value(&mut self.config.log_level, level, format!("{:?}", level))
                        .changed()
                    {
                        self.logger
                            .set_level(self.config.log_level.to_level_filter());
                        self.config.save();
                    }
                }
            });

            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    let buffer = self.log_buffer.lock().unwrap();
                    let buffer_string = buffer.clone();
                    drop(buffer);

                    for message in buffer_string.lines() {
                        if message.trim().is_empty() {
                            continue;
                        }

                        let (level, message) = match message.split_once(" - ") {
                            Some((prefix, message_content)) => {
                                let level = match prefix.split_whitespace().next() {
                                    Some("[ERROR]") => Level::Error,
                                    Some("[WARN]") => Level::Warn,
                                    Some("[INFO]") => Level::Info,
                                    Some("[DEBUG]") => Level::Debug,
                                    Some("[TRACE]") => Level::Trace,
                                    _ => Level::Info,
                                };
                                (level, message_content.to_string())
                            }
                            None => (Level::Info, message.to_string()),
                        };

                        let color = match level {
                            Level::Error => egui::Color32::RED,
                            Level::Warn => egui::Color32::YELLOW,
                            Level::Info => egui::Color32::GREEN,
                            Level::Debug => egui::Color32::LIGHT_BLUE,
                            Level::Trace => egui::Color32::GRAY,
                        };

                        ui.horizontal_wrapped(|ui| {
                            let width = ui.fonts(|f| {
                                f.glyph_width(&TextStyle::Body.resolve(ui.style()), ' ')
                            });
                            ui.spacing_mut().item_spacing.x = width;

                            ui.colored_label(color, format!("[{:?}]", level));
                            ui.label(RichText::new(message).monospace());
                        });
                    }
                });
            ctx.request_repaint();
        });
    }
}
