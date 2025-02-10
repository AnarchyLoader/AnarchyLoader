use egui::{Image, Response, RichText, TextStyle, Ui, Vec2};
use egui_material_icons::icons::{
    ICON_BRAND_AWARENESS, ICON_DESKTOP_WINDOWS, ICON_GROUP, ICON_MENU_BOOK,
    ICON_PRECISION_MANUFACTURING, ICON_PUBLIC, ICON_SEND, ICON_STAR, ICON_SYRINGE, ICON_TIMER,
};

use crate::{
    calculate_session,
    utils::{
        hacks::get_hack_by_dll,
        stats::get_time_from_seconds,
        ui::custom_widgets::{Button, Hyperlink},
    },
    MyApp,
};

#[derive(Debug, Clone)]
pub struct User {
    pub username: String,
    pub avatar_url: String,
}

impl User {
    fn ui(&self, ui: &mut Ui) -> Response {
        let response = ui
            .add(
                Image::new(self.avatar_url.clone())
                    .fit_to_exact_size(Vec2::new(32.0, 32.0))
                    .rounding(8.0)
                    .sense(egui::Sense::click()),
            )
            .on_hover_text(self.username.clone());

        if response.clicked() {
            if let Err(e) = opener::open(format!("https://github.com/{}", self.username)) {
                log::error!("<ABOUT_TAB> {}", format!("Failed to open URL: {}", e));
            }
        }

        response
    }
}

#[derive(Debug, Default, Clone)]
pub struct AboutTab {
    pub is_contributors_parsed: bool,
    pub parsed_contributors: Vec<User>,
    pub is_contributors_loading: bool,
    pub is_stargazers_parsed: bool,
    pub parsed_stargazers: Vec<User>,
    pub is_stargazers_loading: bool,
}

impl MyApp {
    fn fetch_github_users(&mut self, endpoint: &str, user_type: &str) {
        log::info!("<ABOUT_TAB> Parsing {}...", user_type);

        match user_type {
            "contributors" => self.ui.tabs.about.is_contributors_loading = true,
            "stargazers" => self.ui.tabs.about.is_stargazers_loading = true,
            _ => log::error!("<ABOUT_TAB> Unknown user type: {}", user_type),
        }

        let api_url = format!(
            "https://api.github.com/repos/AnarchyLoader/AnarchyLoader/{}?per_page=100",
            endpoint
        );

        let user_type_clone = user_type.to_string();

        let client = ureq::builder().user_agent("AnarchyLoader").build();
        let request = client.get(&api_url);

        match request.call().and_then(|response| {
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

                match &user_type_clone[..] {
                    "contributors" => {
                        self.ui.tabs.about.parsed_contributors = users;
                        self.ui.tabs.about.is_contributors_parsed = true;
                        self.ui.tabs.about.is_contributors_loading = false;
                    }
                    "stargazers" => {
                        self.ui.tabs.about.parsed_stargazers = users;
                        self.ui.tabs.about.is_stargazers_parsed = true;
                        self.ui.tabs.about.is_stargazers_loading = false;
                    }
                    _ => log::error!("<ABOUT_TAB> Unknown user type: {}", user_type),
                }
            }
            Err(e) => {
                log::error!("<ABOUT_TAB> Failed to parse {}: {}", user_type, e);
                match &user_type_clone[..] {
                    "contributors" => self.ui.tabs.about.is_contributors_loading = false,
                    "stargazers" => self.ui.tabs.about.is_stargazers_loading = false,
                    _ => {}
                }
            }
        }
    }

    fn render_user_grid(&mut self, ui: &mut Ui, users: &[User]) {
        for row_users in users.chunks(3) {
            ui.horizontal(|ui| {
                for user in row_users {
                    user.ui(ui);
                }
            });
        }
    }

    pub fn render_about_tab(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .drag_to_scroll(false)
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());


                    ui.vertical_centered(|ui| {
                        ui.add_space(20.0);
                        let image =
                            Image::new(egui::include_image!("../../resources/img/icon.ico"))
                                .max_width(120.0);
                        ui.add(image);
                        ui.add_space(10.0);
                    });


                    ui.vertical_centered(|ui| {
                        ui.heading(RichText::new("AnarchyLoader").text_style(TextStyle::Heading));
                        ui.label(
                            RichText::new(format!("v{}", self.app.meta.version))
                                .text_style(TextStyle::Body),
                        );

                        ui.hyperlink_to(
                            RichText::new(format!("({:.7})", self.app.meta.commit))
                                .monospace()
                                .color(ui.visuals().weak_text_color()),
                            format!(
                                "https://github.com/AnarchyLoader/AnarchyLoader/commit/{}",
                                env!("GIT_HASH")
                            ),
                        );

                        #[cfg(debug_assertions)]
                        {
                            ui.add_space(4.0);
                            ui.colored_label(
                                egui::Color32::GOLD,
                                RichText::new("⚠ DEBUG BUILD ⚠").strong(),
                            );
                        }
                        ui.add_space(10.0);
                    });

                    ui.vertical_centered(|ui| {
                        ui.label(
                            RichText::new("A free and open-source cheat loader for various games.")
                                .text_style(TextStyle::Body)
                                .strong(),
                        );

                        ui.add_space(15.0);


                        if !self.app.config.hide_statistics {
                            ui.group(|ui| {
                                ui.heading("Usage Statistics");

                                if self.app.stats.opened_count == 1 {
                                    ui.colored_label(
                                        egui::Color32::LIGHT_BLUE,
                                        "New user! Welcome!",
                                    );
                                } else {
                                    ui.label(format!(
                                        "Opened {} times",
                                        self.app.stats.opened_count
                                    ));
                                }

                                let mut sorted_inject_counts: Vec<(&String, &u64)> = self
                                    .app
                                    .stats
                                    .inject_counts
                                    .iter()
                                    .collect();

                                if !self.app.stats.has_injections() {
                                    ui.label("No hacks injected yet.");
                                } else {
                                    ui.label(format!(
                                        "Injected {} times",
                                        self.app.stats.inject_counts.values().map(|v| *v).sum::<u64>()
                                    ));

                                    ui.label("Top 3 hacks:");

                                    sorted_inject_counts.sort_by(|a, b| b.1.cmp(a.1));

                                    sorted_inject_counts.iter().take(3).for_each(|(hack_dll, count)| {
                                        ui.label(format!("{}: {}", get_hack_by_dll(&*self.app.hacks, hack_dll).unwrap().name, count));
                                    });
                                }
                            });
                            ui.add_space(10.0);
                        }

                        ui.group(|ui| {
                            ui.heading("Session Information");
                            ui.label(format!("{} OS: {}", ICON_DESKTOP_WINDOWS, &self.app.meta.os_version));
                            ui.label(format!("{} You have been using AnarchyLoader for: {}", ICON_TIMER, &*get_time_from_seconds(self.app.stats.total_seconds.clone())));
                            ui.label(format!("{} Current session: {}", ICON_TIMER, &*calculate_session(self.app.meta.session.clone())));
                        });
                    });

                    ui.add_space(20.0);

                    ui.heading(RichText::new("Quick Links").strong());

                    ui.add_space(5.0);

                    ui.link_button(
                        format!("{} Website", ICON_PUBLIC),
                        "https://anarchy.my",
                        &mut self.toasts,
                    );

                    ui.add_space(5.0);

                    ui.link_button(
                        format!("{} Source Code", ICON_MENU_BOOK),
                        "https://github.com/AnarchyLoader/AnarchyLoader",
                        &mut self.toasts,
                    );

                    ui.add_space(5.0);

                    ui.link_button(
                        format!("{} Injector Code", ICON_SYRINGE),
                        "https://github.com/AnarchyLoader/AnarchyInjector",
                        &mut self.toasts,
                    );

                    ui.add_space(20.0);

                    ui.heading(RichText::new("Social Media").strong());

                    ui.add_space(5.0);

                    ui.link_button(
                        format!("{} Discord", ICON_BRAND_AWARENESS),
                        "https://discord.com/invite/VPGRgXUCsv",
                        &mut self.toasts,
                    );

                    ui.add_space(5.0);

                    ui.link_button(
                        format!("{} Telegram", ICON_SEND),
                        "https://t.me/anarchyloader",
                        &mut self.toasts,
                    );

                    ui.add_space(20.0);
                    let contributors_collapsing = ui.collapsing(
                        RichText::new(format!("{} Contributors", ICON_GROUP)).strong(),
                        |ui| {
                            ui.label("Special thanks to the people who have contributed to this project.");

                            if self.ui.tabs.about.is_contributors_loading {
                                ui.vertical_centered(|ui| ui.label("Loading contributors..."));
                            } else if self.ui.tabs.about.is_contributors_parsed {
                                self.render_user_grid(
                                    ui,
                                    &self.ui.tabs.about.parsed_contributors.clone(),
                                );
                            }
                        },
                    );
                    if contributors_collapsing.fully_open()
                        && !self.ui.tabs.about.is_contributors_parsed
                        && !self.ui.tabs.about.is_contributors_loading
                    {
                        self.fetch_github_users("contributors", "contributors");
                    };

                    ui.add_space(10.0);


                    let stargazers_collapsing = ui.collapsing(
                        RichText::new(format!("{} Stargazers", ICON_STAR)).strong(),
                        |ui| {
                            ui.label("Show your appreciation by starring the project on GitHub!");
                            ui.clink(
                                RichText::new("⭐ Star AnarchyLoader on GitHub").strong(),
                                "https://github.com/AnarchyLoader/AnarchyLoader",
                            );
                            ui.add_space(10.0);

                            if self.ui.tabs.about.is_stargazers_loading {
                                ui.vertical_centered(|ui| ui.label("Loading stargazers..."));
                            } else if self.ui.tabs.about.is_stargazers_parsed {
                                self.render_user_grid(
                                    ui,
                                    &self.ui.tabs.about.parsed_stargazers.clone(),
                                );
                            } else {
                                ui.vertical_centered(|ui| ui.label("Click to load stargazers."));
                            }
                        },
                    );

                    if stargazers_collapsing.fully_open()
                        && !self.ui.tabs.about.is_stargazers_parsed
                        && !self.ui.tabs.about.is_stargazers_loading
                    {
                        self.fetch_github_users("stargazers", "stargazers");
                    };

                    ui.add_space(20.0);


                    ui.heading(RichText::new("Keyboard Shortcuts").strong());
                    egui::Grid::new("keybinds_grid")
                        .num_columns(2)
                        .spacing([20.0, 8.0])
                        .striped(true)
                        .show(ui, |ui| {
                            let keybinds = vec![
                                ("F5", "Refresh hacks list"),
                                ("Enter", "Inject selected hack"),
                                ("Escape", "Deselect hack"),
                                ("Ctrl + Shift", "Toggle Debug Tab"),
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

                    ui.add_space(20.0);


                    ui.vertical_centered(|ui| {
                        ui.horizontal_wrapped(|ui| {
                            let width = ui.fonts(|f| {
                                f.glyph_width(&TextStyle::Body.resolve(ui.style()), ' ')
                            });
                            ui.spacing_mut().item_spacing.x = width;

                            ui.label("Built with");
                            ui.hyperlink_to(
                                format!("{} egui", ICON_PRECISION_MANUFACTURING),
                                "https://www.egui.rs/",
                            );
                            ui.label("by");
                            ui.hyperlink_to("dest4590", "https://github.com/dest4590");
                        });
                        ui.label("© 2025 AnarchyLoader");
                        ui.hyperlink_to("GPL-3.0 License", "https://www.gnu.org/licenses/gpl-3.0");
                    });
                });
        });
    }
}
