use std::{
    collections::BTreeMap, fs, path::Path, process::Command, sync::Arc, thread, time::Duration,
};

use eframe::epaint::{text::TextFormat, FontFamily};
use egui::{
    scroll_area::ScrollBarVisibility::AlwaysHidden, text::LayoutJob, Align,
    CursorIcon::PointingHand as Clickable, FontId, Layout, RichText, Sense, Spinner, TextStyle,
};
use egui_commonmark::CommonMarkViewer;
use egui_material_icons::icons::{
    ICON_AWARD_STAR, ICON_BLOCK, ICON_CHECK, ICON_CLOSE, ICON_CLOUD_OFF, ICON_EDITOR_CHOICE,
    ICON_INVENTORY_2, ICON_LINK, ICON_LOGIN, ICON_MILITARY_TECH, ICON_NO_ACCOUNTS, ICON_PERSON,
    ICON_PROBLEM, ICON_QUESTION_MARK, ICON_SEARCH, ICON_SEARCH_OFF, ICON_STAR, ICON_SYRINGE,
    ICON_VISIBILITY, ICON_WARNING,
};
use egui_modal::Modal;
use url::Url;

use crate::{default_main_menu_message, MyApp};

#[rustfmt::skip]
#[cfg(feature = "scanner")]
use crate::scanner::scanner::Scanner;
use crate::{
    tabs::top_panel::AppTab,
    utils::{
        hacks::{self, Hack},
        ui::custom_widgets::{Button, CheckBox, Hyperlink},
    },
};

#[derive(Debug)]
pub struct HomeTab {
    disclaimer_accepted: bool,
}

impl HomeTab {
    pub fn new() -> Self {
        Self {
            disclaimer_accepted: false,
        }
    }
}

impl Default for HomeTab {
    fn default() -> Self {
        Self::new()
    }
}

impl MyApp {
    // MARK: Key events
    pub fn handle_key_events(&mut self, ctx: &egui::Context) {
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
            log::debug!("<HOME_TAB> Escape key pressed, deselecting hack");
            self.rpc.update(None, Some("Selecting hack"), None);
            self.app.selected_hack = None;
            self.app.config.selected_hack = "".to_string();
            self.ui.text_animator.reset();
        }

        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter)) {
            if let Some(selected) = &self.app.selected_hack {
                log::debug!(
                    "<HOME_TAB> Enter key pressed, injecting hack: {}",
                    selected.name
                );
                self.injection(
                    selected.clone(),
                    ctx.clone(),
                    self.communication.messages.sender.clone(),
                    false,
                );
            }
        }

        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::F5)) {
            log::info!("<HOME_TAB> F5 key pressed, refreshing hacks list");
            self.ui.main_menu_message = "Fetching hacks...".to_string();
            ctx.request_repaint();
            self.app.hacks = match hacks::fetch_hacks(
                &self.app.config.api_endpoint,
                &self.app.config.api_extra_endpoints,
                self.app.config.lowercase_hacks,
            ) {
                Ok(hacks) => {
                    self.ui.main_menu_message = default_main_menu_message();
                    ctx.request_repaint();
                    hacks
                }
                Err(_err) => {
                    self.ui.main_menu_message = "Failed to fetch hacks.".to_string();
                    Vec::new()
                }
            };

            self.toasts.info("Hacks refreshed.");
        }
    }

    pub fn handle_dnd(&mut self, ctx: &egui::Context) {
        let modal = Modal::new(ctx, "dnd_modal");

        modal.show(|ui| {
            ui.heading("Select process:");
            ui.add_space(5.0);
            let dropped_filename = self
                .ui
                .dropped_file
                .path
                .as_ref()
                .and_then(|p| p.file_name())
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            ui.add(
                egui::TextEdit::singleline(&mut self.ui.selected_process_dnd)
                    .hint_text("Enter process name...")
                    .desired_width(200.0),
            );

            ui.add_space(5.0);

            let mut use_x64 = false;

            ui.ccheckbox(&mut use_x64, "Use x64 injector");

            ui.add_space(5.0);

            if ui
                .cbutton(format!("Inject a {}", dropped_filename))
                .clicked()
            {
                if self.ui.selected_process_dnd.is_empty() {
                    self.toasts.error("Please select a process.");
                    return;
                }

                self.toasts.info(format!(
                    "Injecting {} using manual map injection",
                    dropped_filename
                ));

                MyApp::manual_map_inject(
                    self.ui.dropped_file.path.clone(),
                    &self.ui.selected_process_dnd.clone(),
                    self.communication.messages.sender.clone(),
                    self.communication.status_message.clone(),
                    ctx.clone(),
                    use_x64,
                );
                modal.close();
            }
        });

        if let Some(dropped_file) = ctx.input(|i| i.raw.dropped_files.first().cloned()) {
            if dropped_file
                .path
                .as_ref()
                .unwrap()
                .extension()
                .unwrap_or_default()
                != "dll"
            {
                self.toasts.error("Only DLL files are supported.");
                return;
            }
            self.ui.dropped_file = dropped_file.clone();
            modal.open();
        }

        if ctx.input(|i| i.raw.hovered_files.first().is_some()) {
            let screen_rect = ctx.screen_rect();
            let painter = ctx.layer_painter(egui::LayerId::new(
                egui::Order::Foreground,
                egui::Id::new("layer"),
            ));
            painter.rect_filled(screen_rect, 0.0, egui::Color32::from_black_alpha(128));
        }
    }

    // MARK: Home tab
    pub(crate) fn render_home_tab(&mut self, ctx: &egui::Context) {
        self.handle_key_events(ctx);

        let hacks_by_game = MyApp::group_hacks_by_game(&*self.app.hacks, &self.app.config);

        self.render_left_panel(ctx, hacks_by_game);
        self.render_central_panel(ctx);
    }

    pub(crate) fn render_left_panel(
        &mut self,
        ctx: &egui::Context,
        hacks_by_game: BTreeMap<String, BTreeMap<String, Vec<Hack>>>,
    ) {
        egui::SidePanel::left("left_panel")
            .resizable(true)
            .default_width(200.0)
            .max_width(300.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .scroll_bar_visibility(AlwaysHidden)
                    .show(ui, |ui| {
                        ui.style_mut().interaction.selectable_labels = false;

                        ui.add_space(5.0);

                        ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                            ui.add(
                                egui::TextEdit::singleline(&mut self.ui.search_query)
                                    .hint_text(format!("{} Search...", ICON_SEARCH))
                            );
                        });

                        ui.add_space(5.0);
                        let mut all_games_hidden = true;
                        for game_name in self.app.config.game_order.clone() {
                            if let Some(versions) = hacks_by_game.get(&game_name) {
                                if !self.app.config.hidden_games.contains(&game_name) {
                                    self.render_game_hacks(ui, game_name, versions.clone(), ctx);
                                    ui.add_space(5.0);
                                    all_games_hidden = false;
                                }
                            }
                        }
                        if all_games_hidden {
                            if self.app.config.show_only_favorites {
                                ui.label("You enabled\n'Show only favorites' and no favorites are available.");
                                if ui.cbutton(
                                    "Disable 'Show only favorites'",
                                ).clicked() {
                                    self.app.config.show_only_favorites = false;
                                    self.app.config.save();
                                }
                            } else {
                                ui.label("All games are hidden");

                                if ui.cibutton("Show all", ICON_VISIBILITY).clicked() {
                                    self.app.config.hidden_games.clear();
                                    self.app.config.save();
                                }
                            }
                            if ui.cbutton("Go to settings").clicked() {
                                self.ui.tab = AppTab::Settings;
                            }
                        } else if hacks_by_game.is_empty() {
                            ui.label("No hacks available.");
                        }
                    });
            });
    }

    fn render_game_hacks(
        &mut self,
        ui: &mut egui::Ui,
        game_name: String,
        versions: BTreeMap<String, Vec<Hack>>,
        ctx: &egui::Context,
    ) {
        ui.group(|group_ui| {
            group_ui.with_layout(Layout::top_down(Align::Min), |layout_ui| {
                layout_ui.with_layout(Layout::top_down_justified(Align::Center), |ui| {
                    ui.heading(game_name);
                });

                layout_ui.separator();

                for (version, hacks) in versions {
                    self.render_version_hacks(layout_ui, version, hacks, ctx);
                }
            });
        });
    }

    fn render_version_hacks(
        &mut self,
        ui: &mut egui::Ui,
        version: String,
        hacks: Vec<Hack>,
        ctx: &egui::Context,
    ) {
        if !version.is_empty() {
            ui.with_layout(Layout::top_down_justified(Align::Center), |ui| {
                ui.label(RichText::new(version).heading());
            });
        }

        for hack in hacks {
            self.render_hack_item(ui, &hack, ctx);
        }
    }

    fn render_hack_item(&mut self, ui: &mut egui::Ui, hack: &Hack, ctx: &egui::Context) {
        let hack_clone = hack.clone();
        ui.horizontal(|ui| {
            let mut label = self.create_hack_label(hack);

            if !self.ui.search_query.is_empty() {
                label = self.apply_search_highlighting(label, &hack.name);
            }

            let in_progress = self
                .communication
                .in_progress
                .load(std::sync::atomic::Ordering::SeqCst);

            let is_selected = self.app.selected_hack.as_ref() == Some(hack);

            let response = ui
                .add_enabled_ui((!in_progress || is_selected) && hack.working, |ui| {
                    ui.selectable_label(self.app.selected_hack.as_ref() == Some(hack), label)
                        .on_hover_cursor(Clickable)
                })
                .inner;

            // show if hack working
            if !hack.working {
                ui.label(ICON_BLOCK);
            }

            self.render_favorite_button(ui, hack);
            self.render_injection_count(ui, hack);

            if response.clicked() && !in_progress {
                self.select_hack(&hack_clone);
            }

            self.context_menu(&response, ctx, hack);
        });
    }

    fn create_hack_label(&self, hack: &Hack) -> RichText {
        if self.app.config.favorites.contains(&hack.name) {
            RichText::new(&hack.name).color(self.app.config.favorites_color)
        } else {
            RichText::new(&hack.name)
        }
    }

    fn apply_search_highlighting(&self, mut label: RichText, name: &str) -> RichText {
        let lowercase_name = name.to_lowercase();
        let lowercase_query = self.ui.search_query.to_lowercase();
        let mut search_index = 0;
        while let Some(index) = lowercase_name[search_index..].find(&lowercase_query) {
            let start = search_index + index;
            let end = start + lowercase_query.len();
            label = label.strong().underline();
            search_index = end;
        }
        label
    }

    fn render_favorite_button(&mut self, ui: &mut egui::Ui, hack: &Hack) {
        let is_favorite = self.app.config.favorites.contains(&hack.name);
        if is_favorite {
            if ui
                .add(
                    egui::Button::new(ICON_STAR)
                        .frame(false)
                        .sense(Sense::click()),
                )
                .on_hover_cursor(Clickable)
                .clicked()
            {
                self.toggle_favorite(hack.name.clone());
                self.toasts
                    .success(format!("Removed {} from favorites.", hack.name));
            }
        }
    }

    fn toggle_favorite(&mut self, hack_name: String) -> bool {
        if self.app.config.favorites.contains(&hack_name) {
            self.app.config.favorites.remove(&hack_name);
            self.app.config.save();
            false
        } else {
            self.app.config.favorites.insert(hack_name);
            self.app.config.save();
            true
        }
    }

    fn render_injection_count(&self, ui: &mut egui::Ui, hack: &Hack) {
        if self.app.config.hide_statistics {
            return;
        }

        if let Some(&count) = self.app.stats.inject_counts.get(&hack.file) {
            if count == 0 {
                return;
            }

            let label = match count {
                100.. => RichText::new(format!("{}x {}", count, ICON_AWARD_STAR))
                    .color(egui::Color32::YELLOW),
                25.. => format!("{}x {}", count, ICON_EDITOR_CHOICE).into(),
                10.. => format!("{}x {}", count, ICON_MILITARY_TECH).into(),
                _ => format!("{}x", count).into(),
            };

            ui.label(label);
        }
    }

    fn select_hack(&mut self, new_hack: &Hack) {
        log::debug!("<HOME_TAB> Selecting hack: {}", new_hack.name);

        self.ui.text_animator.text = new_hack.name.clone();

        // do not play animation if hack is not changed
        if let Some(selected_hack) = &self.app.selected_hack {
            if selected_hack.name != new_hack.name {
                self.ui.text_animator.reset();
            }
        }

        self.app.selected_hack = Some(new_hack.clone());
        self.app.config.selected_hack = new_hack.name.clone();
        self.app.config.save();

        let mut status = self.communication.status_message.lock().unwrap();
        *status = String::new();

        self.rpc
            .update(None, Some(&format!("Selected {}", new_hack.name)), None);
    }

    // MARK: Hack details
    pub fn display_hack_details(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        selected: &Hack,
    ) {
        let is_roblox = selected.game == "Roblox";

        if self.app.selected_hack.is_some() && self.ui.text_animator.timer < 1.0 {
            self.ui.text_animator.process_animation(ctx);
            ctx.request_repaint();
        }

        ui.vertical(|ui| {
            if !self.app.config.disable_hack_name_animation {
                self.ui.text_animator.render(ui);
            } else {
                ui.label(
                    RichText::new(&selected.name)
                        .size(19.0)
                        .color(self.ui.text_animator.color),
                );
            }
            if !selected.author.is_empty() {
                ui.horizontal_wrapped(|ui| {
                    let width =
                        ui.fonts(|f| f.glyph_width(&TextStyle::Body.resolve(ui.style()), ' '));
                    ui.spacing_mut().item_spacing.x = width;

                    if !selected.local {
                        if selected.author == "???" {
                            ui.label(ICON_NO_ACCOUNTS);
                            ui.label("Unknown author");
                        } else {
                            ui.label(ICON_PERSON);
                            ui.label("by");
                            ui.label(
                                RichText::new(&selected.author).color(self.ui.text_animator.color),
                            )
                                .on_hover_text("Author of the hack");
                        }

                        ui.add_space(5.0);

                        if !selected.source.is_empty() && selected.source.to_string() != "n/a" {
                            if let Ok(url) = Url::parse(&selected.source) {
                                ui.clink(
                                    format!("{} (source, {})", ICON_LINK, url.domain().unwrap()),
                                    &selected.source,
                                );
                            } else {
                                ui.label(format!("{} (cannot parse source)", ICON_CLOUD_OFF));
                            }
                        } else {
                            ui.label(format!("{} (source not available)", ICON_CLOUD_OFF));
                        }
                    } else {
                        ui.label(ICON_INVENTORY_2);
                        ui.label(format!("Local hack ({}, {})", selected.file, selected.arch));
                    }
                });
            }
        });

        ui.separator();

        if !selected.description.is_empty() && !selected.description.contains("n/a") {
            CommonMarkViewer::new().show(ui, &mut self.app.cache, &selected.description);
        } else {
            ui.label(
                RichText::new(format!("{} No description available.", ICON_PROBLEM))
                    .color(self.ui.text_animator.color),
            );
        }

        if !self.app.config.hide_steam_account && !is_roblox {
            ui.horizontal_wrapped(|ui| {
                let width = ui.fonts(|f| f.glyph_width(&TextStyle::Body.resolve(ui.style()), ' '));
                ui.spacing_mut().item_spacing.x = width;

                ui.label(format!("{} Logged in as (Steam):", ICON_LOGIN));
                if ui
                    .label(RichText::new(&self.app.account.name).color(self.ui.text_animator.color))
                    .on_hover_text_at_pointer(&self.app.account.username)
                    .on_hover_cursor(egui::CursorIcon::Help)
                    .clicked()
                {
                    if let Err(e) = opener::open(format!(
                        "https://steamcommunity.com/profiles/{}/",
                        self.app.account.id
                    )) {
                        self.toasts
                            .error(format!("Failed to open Steam profile: {}", e));
                    }
                }
            });
        }

        let modal = Modal::new(ctx, "disclaimer");

        modal.show(|ui| {
            ui.heading(RichText::new(format!("{} Disclaimer", ICON_WARNING)).color(egui::Color32::RED));
            ui.separator();

            let mut job = LayoutJob::default();

            let text_format = TextFormat { font_id: FontId::new(14.0, FontFamily::Proportional), ..Default::default() };

            job.append(&format!("Hey {}!", whoami::username()), 0.0, text_format.clone());
            job.append("\n\n", 0.0, TextFormat::default());
            job.append("Using cheats or unauthorized modifications in online games violates their terms of service.", 0.0, text_format.clone());
            job.append("\n\n", 0.0, TextFormat::default());
            job.append("By using this tool, you understand and agree that you are doing so at your own risk.", 0.0, text_format.clone());
            job.append("\n\n", 0.0, TextFormat::default());
            job.append("This may result in a ", 0.0, text_format.clone());
            job.append("permanent ban", 0.0, TextFormat {
                color: egui::Color32::RED,
                ..Default::default()
            });
            job.append(" from the game and related services.", 0.0, text_format.clone());
            job.append("\n\n", 0.0, TextFormat::default());
            job.append("We are not responsible for any consequences resulting from the use of this program.", 0.0, TextFormat {
                color: self.ui.text_animator.color,
                ..Default::default()
            });

            ui.label(job);

            ui.add_space(5.0);

            ui.horizontal(|ui| {
                if ui.cibutton("I understand", ICON_CHECK).clicked() {
                    self.ui.tabs.home.disclaimer_accepted = true;
                    self.toasts.info("Press the inject button again to proceed.");
                    modal.close();
                }
                if ui.cibutton("Cancel", ICON_CLOSE).clicked() {
                    modal.close();
                    self.toasts.custom("But why did you download the loader?", ICON_QUESTION_MARK.to_string(), egui::Color32::from_rgb(150, 200, 210));
                    return;
                }
            });
        });

        // MARK: Inject button
        let is_32bit = size_of::<usize>() == 4;
        let is_cs2_32bit = is_32bit && selected.process == "cs2.exe";
        let in_progress = self
            .communication
            .in_progress
            .load(std::sync::atomic::Ordering::SeqCst);
        let inject_button = ui
            .add_enabled_ui(!in_progress && !is_cs2_32bit, |ui| {
                ui.button_with_tooltip(
                    if !is_roblox {
                        format!("{} Inject {}", ICON_SYRINGE, selected.name)
                    } else {
                        "Run".to_string()
                    },
                    &selected.file,
                )
            })
            .inner;

        if is_32bit {
            ui.label(
                RichText::new("32-bit detected, cs2 hacks are not supported.")
                    .color(egui::Color32::RED),
            );
        }

        if inject_button.clicked() && !is_cs2_32bit {
            if !self.ui.tabs.home.disclaimer_accepted && !self.app.stats.has_injections() {
                modal.open();
                return;
            }

            self.toasts
                .custom(
                    if !is_roblox {
                        format!("Injecting {}", selected.name)
                    } else {
                        "Running...".to_string()
                    },
                    "⌛".to_string(),
                    egui::Color32::from_rgb(150, 200, 210),
                )
                .duration(Some(Duration::from_secs(2)));

            self.rpc.update(
                None,
                Some(&if !is_roblox {
                    format!("Injecting {}", selected.name)
                } else {
                    "Running...".to_string()
                }),
                Some(if !is_roblox { "injecting" } else { "running" }),
            );

            if !is_roblox {
                self.injection(
                    selected.clone(),
                    ctx.clone(),
                    self.communication.messages.sender.clone(),
                    if ctx.input(|i| i.modifiers.ctrl) {
                        true
                    } else {
                        false
                    },
                );
            } else {
                self.run_executor(
                    selected.clone(),
                    ctx.clone(),
                    self.communication.messages.sender.clone(),
                );
            }
        }

        if in_progress {
            ui.add_space(5.0);
            let status = self.communication.status_message.lock().unwrap().clone();
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.add(Spinner::new());
                    ui.add_space(5.0);
                    ui.label(
                        RichText::new(&status).color(if status.starts_with("Failed") {
                            egui::Color32::RED
                        } else {
                            self.ui.text_animator.color
                        }),
                    );
                    ctx.request_repaint();
                });
            });
        } else {
            ui.add_space(5.0);
            let status = self.communication.status_message.lock().unwrap().clone();
            if !status.is_empty() {
                let text_color = if status.starts_with("Failed") || status.starts_with("Error") {
                    egui::Color32::RED
                } else {
                    self.ui.text_animator.color
                };

                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        let s = if status.contains("Can not find process") {
                            ICON_SEARCH_OFF.to_owned() + &*status
                        } else {
                            status
                        }
                            .replace(
                                "Failed to execute injector: ",
                                "Failed to execute injector:\n",
                            );

                        ui.label(RichText::new(&s).color(text_color));
                    });
                });
            }
        }
    }

    pub fn context_menu(&mut self, response: &egui::Response, ctx: &egui::Context, hack: &Hack) {
        // MARK: Context menu
        let file_path_owned = hack.file_path.clone();
        let ctx_clone = ctx.clone();
        let status_message = Arc::clone(&self.communication.status_message);
        let is_favorite = self.app.config.favorites.contains(&hack.name);
        let is_roblox = hack.game == "Roblox";

        response.context_menu(|ui| {
            if is_favorite {
                if ui.cbutton("Remove from favorites").clicked() {
                    self.app.config.favorites.remove(&hack.name);
                    self.app.config.save();
                    self.toasts
                        .success(format!("Removed {} from favorites.", hack.name));
                    ui.close_menu();
                }
            } else {
                if ui.cbutton("Add to favorites").clicked() {
                    self.app.config.favorites.insert(hack.name.clone());
                    self.app.config.save();
                    self.toasts
                        .success(format!("Added {} to favorites.", hack.name));
                    ui.close_menu();
                }
            }

            if !is_roblox && !hack.local {
                // show only if file exists
                if Path::new(&file_path_owned).exists() {
                    if ui
                        .button_with_tooltip(
                            "Open in Explorer",
                            "Open the file location in Explorer",
                        )
                        .clicked()
                    {
                        if let Err(e) = Command::new("explorer.exe")
                            .arg(format!("/select,{}", hack.file_path.to_string_lossy()))
                            .spawn()
                        {
                            let mut status = self.communication.status_message.lock().unwrap();
                            *status = format!("Failed to open Explorer: {}", e);
                            self.toasts.error(format!("Failed to open Explorer: {}", e));
                        }
                        ui.close_menu();
                    }

                    if ui
                        .button_with_tooltip("Uninstall", "Uninstall the selected hack")
                        .clicked()
                    {
                        if let Err(e) = fs::remove_file(&file_path_owned) {
                            let mut status = self.communication.status_message.lock().unwrap();
                            *status = format!("Failed to uninstall: {}", e);
                        } else {
                            let mut status = self.communication.status_message.lock().unwrap();
                            *status = "Uninstall successful.".to_string();
                        }
                        ui.close_menu();
                    }

                    if ui
                        .button_with_tooltip("Reinstall", "Reinstall the selected hack")
                        .clicked()
                    {
                        let hack_clone = hack.clone();
                        thread::spawn(move || {
                            if !Path::new(&file_path_owned).exists() {
                                let mut status = status_message.lock().unwrap();
                                *status = "Failed to reinstall: file does not exist.".to_string();
                                ctx_clone.request_repaint();
                                return;
                            }
                            {
                                let mut status = status_message.lock().unwrap();
                                *status = "Reinstalling...".to_string();
                                ctx_clone.request_repaint();
                            }
                            if let Err(e) = fs::remove_file(&file_path_owned) {
                                let mut status = status_message.lock().unwrap();
                                *status = format!("Failed to delete file: {}", e);
                                ctx_clone.request_repaint();
                                return;
                            }
                            match hack_clone.download(file_path_owned.to_string_lossy().to_string())
                            {
                                Ok(_) => {
                                    let mut status = status_message.lock().unwrap();
                                    *status = "Reinstalled.".to_string();
                                    ctx_clone.request_repaint();
                                }
                                Err(e) => {
                                    let mut status = status_message.lock().unwrap();
                                    *status = format!("Failed to reinstall: {}", e);
                                    ctx_clone.request_repaint();
                                }
                            }
                        });
                        ui.close_menu();
                    }

                    #[cfg(feature = "scanner")]
                    if ui
                        .button_with_tooltip("Scan", "Scan the selected hack")
                        .clicked()
                    {
                        let scanner =
                            Scanner::new(std::path::PathBuf::from(hack.file_path.clone()));

                        match scanner.scan(self.app.meta.path.clone()) {
                            Ok(()) => {
                                self.open_scanner_log();
                            }
                            Err(err) => {
                                self.toasts.error(err);
                            }
                        }
                        ui.close_menu();
                    }
                } else {
                    if ui.cbutton("Download").clicked() {
                        let file_path = hack.file_path.clone();
                        let hack_clone = hack.clone();
                        thread::spawn(move || {
                            match hack_clone.download(file_path.to_string_lossy().to_string()) {
                                Ok(_) => {
                                    let mut status = status_message.lock().unwrap();
                                    *status = "Downloaded.".to_string();
                                }
                                Err(e) => {
                                    let mut status = status_message.lock().unwrap();
                                    *status = format!("Failed to download: {}", e);
                                }
                            }
                        });
                        ui.close_menu();
                    }
                }
            }

            if hack.local {
                if ui.cbutton("Remove").clicked() {
                    self.app.config.local_hacks.retain(|h| {
                        Path::new(&h.dll)
                            .file_name()
                            .map_or(true, |f| f != hack.file_path.file_name().unwrap())
                    });
                    self.app.config.save();
                    let grouped =
                        MyApp::group_hacks_by_game_internal(&self.app.hacks, &self.app.config);
                    self.app.config.game_order = grouped.keys().cloned().collect();
                    self.toasts.success(format!("Removed {}.", hack.name));
                    ui.close_menu();
                }
            }
        });
    }
}
