use egui::{Image, InnerResponse, Response, RichText, TextStyle, Ui, Vec2, Widget};
use egui_material_icons::icons::{
    ICON_BRAND_AWARENESS, ICON_COMPUTER, ICON_MENU_BOOK, ICON_PRECISION_MANUFACTURING, ICON_PUBLIC,
    ICON_SEND, ICON_SYRINGE, ICON_TIMER,
};

use crate::{
    calculate_session,
    utils::custom_widgets::{Button, Hyperlink},
    MyApp,
};

#[derive(Debug, Clone)]
pub struct User {
    pub username: String,
    pub avatar_url: String,
}

impl User {
    fn ui(&self, ui: &mut Ui) -> Response {
        ui.add(
            Image::new(self.avatar_url.clone())
                .fit_to_exact_size(Vec2::new(32.0, 32.0))
                .rounding(8.0),
        )
        .on_hover_text(self.username.clone())
    }
}

#[derive(Debug, Default, Clone)]
pub struct AboutTab {
    pub is_contributors_parsed: bool,
    pub parsed_contributors: Vec<User>,
    pub is_stargazers_parsed: bool,
    pub parsed_stargazers: Vec<User>,
}

impl MyApp {
    fn fetch_github_users(&mut self, endpoint: &str, user_type: &str) {
        log::info!("Parsing {}...", user_type);
        let api_url = format!(
            "https://api.github.com/repos/AnarchyLoader/AnarchyLoader/{}?per_page=100",
            endpoint
        );

        match ureq::get(&api_url).call().and_then(|response| {
            response
                .into_json::<Vec<serde_json::Value>>()
                .map_err(ureq::Error::from)
        }) {
            Ok(users_data) => {
                let users: Vec<User> = users_data
                    .into_iter()
                    .map(|user_json| User {
                        username: user_json["login"].as_str().unwrap().to_string(),
                        avatar_url: user_json["avatar_url"].as_str().unwrap().to_string(),
                    })
                    .collect();

                match user_type {
                    "contributors" => {
                        self.ui.tab_states.about.parsed_contributors = users;
                        self.ui.tab_states.about.is_contributors_parsed = true;
                    }
                    "stargazers" => {
                        self.ui.tab_states.about.parsed_stargazers = users;
                        self.ui.tab_states.about.is_stargazers_parsed = true;
                    }
                    _ => log::error!("Unknown user type: {}", user_type),
                }
            }
            Err(e) => log::error!("Failed to parse {}: {}", user_type, e),
        }
    }

    fn render_user_grid(&mut self, ui: &mut Ui, users: &[User]) {
        let mut count = 0;
        ui.horizontal(|ui| {
            for user in users {
                user.ui(ui);
                count += 1;
                if count % 3 == 0 {
                    ui.end_row();
                }
            }
            if count % 3 != 0 {
                ui.end_row();
            }
        });
    }

    pub fn render_about_tab(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .drag_to_scroll(false)
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());

                    // Logo section
                    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                        let image = Image::new(egui::include_image!("../../resources/img/icon.ico"))
                            .max_width(100.0);

                        ui.add(image);
                    });

                    // Version info
                    ui.vertical_centered(|ui| {
                        ui.add_space(10.0);
                        ui.label(
                            RichText::new(format!("v{}", self.app.meta.version))
                                .text_style(TextStyle::Heading)
                        );

                        ui.hyperlink_to(
                            RichText::new(format!("({:.7})", self.app.meta.commit))
                                .monospace()
                                .color(ui.visuals().weak_text_color()),
                            format!("https://github.com/AnarchyLoader/AnarchyLoader/commit/{}", env!("GIT_HASH")),
                        );

                        #[cfg(debug_assertions)]
                        {
                            ui.add_space(4.0);
                            ui.colored_label(
                                egui::Color32::GOLD,
                                RichText::new("⚠ DEBUG BUILD ⚠").strong(),
                            );
                        }
                    });

                    ui.add_space(5.0);

                    // Main content
                    ui.vertical_centered(|ui| {
                        ui.label(
                            RichText::new("AnarchyLoader is a free and open-source cheat loader for various games.")
                                .text_style(TextStyle::Body)
                                .strong()
                        );

                        ui.add_space(10.0);

                        // Statistics
                        if !self.app.config.hide_statistics {
                            ui.horizontal(|ui| {
                                ui.label("📊 Statistics:");
                                if self.app.statistics.opened_count == 1 {
                                    ui.colored_label(egui::Color32::LIGHT_BLUE, "New user! Welcome!");
                                } else {
                                    ui.label(format!("Opened {} times", self.app.statistics.opened_count));
                                }
                            });
                        }

                        // System info
                        ui.horizontal(|ui| {
                            ui.label(format!("{} OS:", ICON_COMPUTER));
                            ui.label(&self.app.meta.os_version);
                        });

                        ui.horizontal(|ui| {
                            ui.label(format!("{} Session:", ICON_TIMER));
                            ui.label("Your session was running for: ".to_string() + &*calculate_session(self.app.meta.session.clone()));
                        });
                    });

                    ui.add_space(15.0);

                    // Links
                    ui.heading("Links");
                    ui.link_button(format!("{} Website", ICON_PUBLIC), "https://anarchy.my", &mut self.toasts);
                    ui.add_space(5.0);
                    ui.link_button(format!("{} Source Code", ICON_MENU_BOOK), "https://github.com/AnarchyLoader/AnarchyLoader", &mut self.toasts);
                    ui.add_space(5.0);
                    ui.link_button(format!("{} Injector Code", ICON_SYRINGE), "https://github.com/AnarchyLoader/AnarchyInjector", &mut self.toasts);
                    ui.add_space(15.0);

                    // Socials
                    ui.heading("Social Media");
                    ui.link_button(format!("{} Discord", ICON_BRAND_AWARENESS), "https://discord.com/invite/VPGRgXUCsv", &mut self.toasts);
                    ui.add_space(5.0);
                    ui.link_button(format!("{} Telegram", ICON_SEND), "https://t.me/anarchyloader", &mut self.toasts);
                    ui.add_space(15.0);

                    // Contributors
                    let contributors_collapsing = ui.collapsing("Contributors", |ui| {
                        ui.label("Special thanks to all the contributors who helped make this project possible.");
                        ui.add_space(5.0);
                        if self.ui.tab_states.about.is_contributors_parsed {
                            self.render_user_grid(ui, &self.ui.tab_states.about.parsed_contributors.clone());
                        }
                    });
                    if contributors_collapsing.fully_open() && !self.ui.tab_states.about.is_contributors_parsed {
                        self.fetch_github_users("contributors", "contributors");
                    };

                    ui.add_space(5.0);

                    // Stargazers
                    let stargazers_collapsing = ui.collapsing("Stargazers", |ui| {
                        ui.label("Show your support by starring the project on GitHub!");
                        ui.clink("Star AnarchyLoader!", "https://github.com/AnarchyLoader/AnarchyLoader");
                        ui.add_space(5.0);

                        if self.ui.tab_states.about.is_stargazers_parsed {
                            self.render_user_grid(ui, &self.ui.tab_states.about.parsed_stargazers.clone());
                        }
                    });

                    if stargazers_collapsing.fully_open() && !self.ui.tab_states.about.is_stargazers_parsed {
                        self.fetch_github_users("stargazers", "stargazers");
                    };

                    ui.add_space(15.0);

                    // Keybinds
                    ui.heading("Keyboard Shortcuts");
                    egui::Grid::new("keybinds_grid")
                        .num_columns(2)
                        .spacing([20.0, 4.0])
                        .striped(true)
                        .show(ui, |ui| {
                            let keybinds = vec![
                                ("F5", "Refresh hacks"),
                                ("Enter", "Inject selected hack"),
                                ("Escape", "Deselect hack"),
                                ("Ctrl + Shift", "Show debug tab"),
                            ];

                            for (key, action) in keybinds {
                                ui.colored_label(
                                    ui.visuals().strong_text_color(),
                                    RichText::new(key).monospace(),
                                );
                                ui.label(action);
                                ui.end_row();
                            }
                        });

                    ui.add_space(10.0);

                    // Footer
                    ui.vertical_centered(|ui| {
                        ui.horizontal_wrapped(|ui| {
                            let width =
                                ui.fonts(|f| f.glyph_width(&TextStyle::Body.resolve(ui.style()), ' '));
                            ui.spacing_mut().item_spacing.x = width;

                            ui.label("Built with");
                            ui.hyperlink_to(format!("{} egui", ICON_PRECISION_MANUFACTURING), "https://www.egui.rs/");
                            ui.label("by");
                            ui.hyperlink_to("dest4590", "https://github.com/dest4590");
                        });
                        ui.label("© 2025 AnarchyLoader. Open source under GPL-3.0 License");
                    });
                });
        });
    }
}
