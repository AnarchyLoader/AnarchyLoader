use egui::{RichText, Sense, TextStyle};

use crate::{
    custom_widgets::{Button, Hyperlink},
    MyApp,
};

impl MyApp {
    pub fn render_about_tab(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .drag_to_scroll(false)
                .show(ui, |ui| {
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
                    ui.add_space(5.0);
                    ui.label(RichText::new("AnarchyLoader is a free and open-source cheat loader for various games.").size(16.0));
                    ui.label(format!("btw, you opened it {} times", self.app.statistics.opened_count));
                    ui.add_space(5.0);
                    ui.horizontal_wrapped(|ui| {
                        let width =
                            ui.fonts(|f| f.glyph_width(&TextStyle::Body.resolve(ui.style()), ' '));
                        ui.spacing_mut().item_spacing.x = width;

                        ui.clink("Made with egui", "https://www.github.com/emilk/egui/");
                        ui.clink("by dest4590", "https://github.com/dest4590");
                    });
                    ui.add_space(5.0);
                    ui.horizontal(|ui| {
                        ui.link_button("Visit Website", "https://anarchy.my", &mut self.toasts);
                        ui.link_button(
                            "Source code",
                            "https://github.com/AnarchyLoader/AnarchyLoader",
                            &mut self.toasts,
                        );
                        ui.link_button(
                            "Injector source code",
                            "https://github.com/AnarchyLoader/AnarchyInjector",
                            &mut self.toasts,
                        );
                    });

                    ui.add_space(5.0);
                    ui.heading("Socials:");
                    ui.add_space(5.0);

                    ui.horizontal(|ui| {
                        ui.link_button(
                            "Discord",
                            "https://discord.com/invite/YzwcDCsRe8",
                            &mut self.toasts,
                        );
                        ui.link_button("Telegram", "https://t.me/anarchyloader", &mut self.toasts);
                    });

                    ui.add_space(5.0);

                    ui.heading("Keybinds:");

                    let keybinds = vec![
                        ("F5", "Refresh hacks"),
                        ("Enter", "Inject selected hack"),
                        ("Escape", "Deselect hack"),
                        ("Hold Shift", "Debug tab"),
                    ];

                    for (key, action) in &keybinds {
                        ui.label(format!("{}: {}", key, action));
                    }
                });
        });
    }
}
