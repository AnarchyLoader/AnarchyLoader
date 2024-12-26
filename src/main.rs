#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod custom_widgets;
mod hacks;
mod inject;
mod tabs;
mod utils;

use std::{
    collections::BTreeMap,
    env,
    sync::{
        mpsc::{self, Receiver, Sender, TryRecvError},
        Arc, Mutex, OnceLock,
    },
    time::Duration,
};

use eframe::{
    egui::{self, RichText},
    App,
};
use egui::{
    scroll_area::ScrollBarVisibility::AlwaysHidden, Color32, CursorIcon::PointingHand as Clickable,
    DroppedFile, Frame, Margin, Sense,
};
use egui_alignments::center_vertical;
use egui_notify::Toasts;
use hacks::{get_all_processes, get_hack_by_name, Hack};
use tabs::top_panel::AppTab;
use utils::{
    config::Config, logger::MyLogger, rpc::Rpc, statistics::Statistics, steam::SteamAccount,
};

pub(crate) fn load_icon() -> egui::IconData {
    let (icon_rgba, icon_width, icon_height) = {
        let icon = include_bytes!("../resources/img/icon.ico");
        let image = image::load_from_memory(icon)
            .expect("Failed to open icon path")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };

    egui::IconData {
        rgba: icon_rgba,
        width: icon_width,
        height: icon_height,
    }
}

fn main() {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_min_inner_size(egui::vec2(600.0, 200.0))
            .with_inner_size(egui::vec2(800.0, 400.0))
            .with_icon(std::sync::Arc::new(load_icon())),
        ..Default::default()
    };
    eframe::run_native(
        "AnarchyLoader",
        native_options,
        Box::new(|cc| Ok(Box::new(MyApp::new(cc)))),
    )
    .unwrap();
}

struct MyApp {
    hacks: Vec<Hack>,
    hacks_processes: Vec<String>,
    selected_hack: Option<Hack>,
    status_message: Arc<Mutex<String>>,
    parse_error: Option<String>,
    app_version: String,
    inject_in_progress: Arc<std::sync::atomic::AtomicBool>,
    tab: AppTab,
    search_query: String,
    main_menu_message: String,
    config: Config,
    statistics: Statistics,
    toasts: Toasts,
    message_sender: Sender<String>,
    message_receiver: Receiver<String>,
    dropped_file: DroppedFile,
    selected_process_dnd: String,
    account: SteamAccount,
    rpc: Rpc,
    log_buffer: Arc<Mutex<String>>,
    logger: MyLogger,
}

fn default_main_menu_message() -> String {
    format!(
        "Hello {}!\nPlease select a cheat from the list.",
        whoami::username()
    )
}

static LOGGER: OnceLock<MyLogger> = OnceLock::new();

impl MyApp {
    // MARK: Init
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let config = Config::load();

        let logger = MyLogger::init();
        let log_buffer = logger.buffer.clone();

        log::set_max_level(config.log_level.to_level_filter());

        log::info!("AnarchyLoader v{}", env!("CARGO_PKG_VERSION"));

        let (message_sender, message_receiver) = mpsc::channel();
        let mut statistics = Statistics::load();

        statistics.increment_opened_count();

        let status_message = Arc::new(Mutex::new(String::new()));
        let inject_in_progress = Arc::new(std::sync::atomic::AtomicBool::new(false));

        let hacks = hacks::Hack::fetch_hacks(&config.api_endpoint, config.lowercase_hacks)
            .unwrap_or_default();

        let hacks_processes = get_all_processes(&hacks);

        let account = match SteamAccount::new() {
            Ok(account) => account,
            Err(_) => SteamAccount::default(),
        };

        let rpc = Rpc::new();
        rpc.update(
            Some(&format!("v{}", env!("CARGO_PKG_VERSION"))),
            Some("Selecting a hack"),
        );

        let mut selected_hack = None;

        if config.selected_hack != "" && config.automatically_select_hack {
            selected_hack = get_hack_by_name(&hacks, &config.selected_hack);
        }

        Self {
            hacks,
            hacks_processes,
            selected_hack,
            status_message,
            parse_error: None,
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            inject_in_progress,
            tab: AppTab::default(),
            search_query: String::new(),
            main_menu_message: default_main_menu_message(),
            config,
            statistics,
            toasts: Toasts::default(),
            message_sender,
            message_receiver,
            dropped_file: DroppedFile::default(),
            selected_process_dnd: String::new(),
            account,
            rpc,
            log_buffer,
            logger: logger.clone(),
        }
    }

    // MARK: Home tab
    fn render_home_tab(&mut self, ctx: &egui::Context, theme_color: egui::Color32) {
        match self.message_receiver.try_recv() {
            Ok(message) => {
                if message.starts_with("SUCCESS: ") {
                    let name = message.trim_start_matches("SUCCESS: ").to_string();
                    self.toasts
                        .success(format!("Successfully injected {}", name))
                        .duration(Some(Duration::from_secs(4)));

                    self.statistics.increment_inject_count(&name);
                } else {
                    self.toasts
                        .error(message)
                        .duration(Some(Duration::from_secs(4)));
                }

                self.rpc.update(
                    Some(&format!("v{}", env!("CARGO_PKG_VERSION"))),
                    Some("Selecting a hack"),
                );
            }
            Err(TryRecvError::Empty) => {}
            Err(e) => {
                log::error!("Error receiving from channel: {:?}", e);
            }
        }

        self.handle_key_events(ctx);

        let mut hacks_by_game: BTreeMap<String, BTreeMap<String, Vec<Hack>>> = BTreeMap::new();

        for hack in self.hacks.clone() {
            if self.config.show_only_favorites && !self.config.favorites.contains(&hack.name) {
                continue;
            }

            let game = hack.game.clone();
            if game.starts_with("CSS") {
                let mut parts = game.split_whitespace();
                let game_name = parts.next().unwrap_or("CSS").to_string();
                let version = parts.collect::<Vec<&str>>().join(" ");
                let version = if version.is_empty() {
                    "Unknown version".to_string()
                } else {
                    version
                };
                hacks_by_game
                    .entry(game_name)
                    .or_insert_with(BTreeMap::new)
                    .entry(version)
                    .or_insert_with(Vec::new)
                    .push(hack);
            } else {
                hacks_by_game
                    .entry(game.clone())
                    .or_insert_with(BTreeMap::new)
                    .entry("".to_string())
                    .or_insert_with(Vec::new)
                    .push(hack);
            }
        }

        // MARK: Left panel
        egui::SidePanel::left("left_panel")
            .resizable(true)
            .default_width(200.0)
            .max_width(300.0)
            .frame(
                Frame::default()
                    .fill(Color32::from_rgb(27, 27, 27))
                    .inner_margin(Margin::symmetric(10.0, 10.0)),
            )
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .scroll_bar_visibility(AlwaysHidden)
                    .show(ui, |ui| {
                        ui.style_mut().interaction.selectable_labels = false;

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                            ui.add(
                                egui::TextEdit::singleline(&mut self.search_query)
                                    .hint_text("Search..."),
                            );
                        });

                        ui.add_space(5.0);

                        for (game_name, versions) in hacks_by_game {
                            ui.group(|ui| {
                                ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                                    ui.with_layout(
                                        egui::Layout::top_down_justified(egui::Align::Center),
                                        |ui| ui.heading(game_name),
                                    );
                                    ui.separator();
                                    for (version, hacks) in versions {
                                        if !version.is_empty() {
                                            ui.with_layout(
                                                egui::Layout::top_down_justified(
                                                    egui::Align::Center,
                                                ),
                                                |ui| ui.label(RichText::new(version).heading()),
                                            );
                                        }

                                        for hack in hacks {
                                            let hack_clone = hack.clone();
                                            ui.horizontal(|ui| {
                                                let is_favorite =
                                                    self.config.favorites.contains(&hack.name);

                                                let mut label = if is_favorite {
                                                    RichText::new(&hack.name)
                                                        .color(self.config.favorites_color)
                                                } else {
                                                    RichText::new(&hack.name)
                                                };

                                                if !self.search_query.is_empty() {
                                                    let lowercase_name = hack.name.to_lowercase();
                                                    let lowercase_query =
                                                        self.search_query.to_lowercase();
                                                    let mut search_index = 0;
                                                    while let Some(index) = lowercase_name
                                                        [search_index..]
                                                        .find(&lowercase_query)
                                                    {
                                                        let start = search_index + index;
                                                        let end = start + lowercase_query.len();
                                                        label = label.strong().underline();
                                                        search_index = end;
                                                    }
                                                }

                                                let response = ui.selectable_label(
                                                    self.selected_hack.as_ref() == Some(&hack),
                                                    label,
                                                );

                                                if is_favorite {
                                                    let favorite_icon = "★";
                                                    if ui
                                                        .add(
                                                            egui::Button::new(RichText::new(
                                                                favorite_icon,
                                                            ))
                                                            .frame(false)
                                                            .sense(Sense::click()),
                                                        )
                                                        .on_hover_cursor(Clickable)
                                                        .clicked()
                                                    {
                                                        if is_favorite {
                                                            self.config
                                                                .favorites
                                                                .remove(&hack.name);
                                                        } else {
                                                            self.config
                                                                .favorites
                                                                .insert(hack.name.clone());
                                                        }
                                                        self.config.save();
                                                    }
                                                }

                                                if !self.config.hide_statistics {
                                                    let count = self
                                                        .statistics
                                                        .inject_counts
                                                        .get(&hack.name)
                                                        .unwrap_or(&0);
                                                    if count != &0 {
                                                        ui.label(format!("{}x", count));
                                                    }
                                                }

                                                if response.clicked() {
                                                    self.selected_hack = Some(hack_clone.clone());

                                                    self.config.selected_hack =
                                                        hack_clone.name.clone();
                                                    self.config.save();

                                                    let mut status =
                                                        self.status_message.lock().unwrap();
                                                    *status = String::new();

                                                    self.rpc.update(
                                                        None,
                                                        Some(&format!(
                                                            "Selected {}",
                                                            hack_clone.name
                                                        )),
                                                    );
                                                }

                                                self.context_menu(&response, ctx, &hack);

                                                response.on_hover_cursor(Clickable);
                                            });
                                        }
                                    }
                                });
                            });
                            ui.add_space(5.0);
                        }
                    });
            });

        // MARK: Selected hack panel
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(selected) = self.selected_hack.clone() {
                self.display_hack_details(ui, ctx, &selected, theme_color);
            } else {
                center_vertical(ui, |ui| {
                    ui.label(self.main_menu_message.clone());
                });
            }
        });
    }
}

impl App for MyApp {
    // MARK: Global render
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui_extras::install_image_loaders(ctx);

        let is_dark_mode = ctx.style().visuals.dark_mode;
        let theme_color = if is_dark_mode {
            egui::Color32::LIGHT_GRAY
        } else {
            egui::Color32::DARK_GRAY
        };

        if self.parse_error.is_some() {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(130.0);
                    ui.colored_label(
                        egui::Color32::RED,
                        RichText::new(self.parse_error.as_ref().unwrap())
                            .size(24.0)
                            .strong(),
                    );

                    ui.label("API Endpoint (editable):");
                    if ui
                        .text_edit_singleline(&mut self.config.api_endpoint)
                        .changed()
                    {
                        self.config.save();
                    }
                });
            });
            return;
        }

        self.render_top_panel(ctx);

        self.handle_dnd(ctx);

        match self.tab {
            AppTab::Home => self.render_home_tab(ctx, theme_color),
            AppTab::Settings => self.render_settings_tab(ctx),
            AppTab::About => self.render_about_tab(ctx),
            AppTab::Logs => self.render_logs_tab(ctx),
            AppTab::Debug => self.render_debug_tab(ctx),
        }
    }
}
