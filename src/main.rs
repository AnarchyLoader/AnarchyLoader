#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod games;
mod hacks;
mod inject;
mod scanner;
mod tabs;
mod utils;

use std::{
    collections::BTreeMap,
    env,
    sync::{Arc, Mutex, OnceLock},
};

use eframe::{
    egui::{self, RichText},
    App,
};
use egui::{
    emath::easing, scroll_area::ScrollBarVisibility::AlwaysHidden,
    CursorIcon::PointingHand as Clickable, DroppedFile, Id, Sense,
};
use egui_alignments::center_vertical;
use egui_commonmark::CommonMarkCache;
use egui_material_icons::icons::{
    ICON_AWARD_STAR, ICON_EDITOR_CHOICE, ICON_MILITARY_TECH, ICON_VISIBILITY,
};
use egui_notify::Toasts;
use egui_transition_animation::{animated_pager, TransitionStyle, TransitionType};
use games::local::LocalUI;
use hacks::{get_all_processes, get_hack_by_name, Hack};
use is_elevated::is_elevated;
#[cfg(feature = "scanner")]
use scanner::scanner::ScannerPopup;
use tabs::{settings::TransitionPopup, top_panel::AppTab};
use utils::{
    config::Config,
    custom_widgets::{Button, CheckBox, Hyperlink},
    logger::MyLogger,
    messages::ToastsMessages,
    rpc::{Rpc, RpcUpdate},
    statistics::Statistics,
    steam::SteamAccount,
    updater::Updater,
};
use winreg::{enums::HKEY_LOCAL_MACHINE, RegKey};

use crate::{
    tabs::about::AboutTab,
    utils::intro::{AnimationPhase, AnimationState},
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
            .with_icon(Arc::new(load_icon())),
        ..Default::default()
    };
    eframe::run_native(
        if !is_elevated() {
            "AnarchyLoader"
        } else {
            "AnarchyLoader (Administrator)"
        },
        native_options,
        Box::new(|cc| Ok(Box::new(MyApp::new(cc)))),
    )
    .unwrap();
}

#[derive(Debug)]
struct AppState {
    hacks: Vec<Hack>,
    hacks_processes: Vec<String>,
    selected_hack: Option<Hack>,
    config: Config,
    stats: Statistics,
    account: SteamAccount,
    updater: Updater,
    cache: CommonMarkCache,
    meta: AppMeta,
}

#[derive(Debug)]
struct UIState {
    tab: AppTab,
    tab_states: TabStates,
    search_query: String,
    main_menu_message: String,
    dropped_file: DroppedFile,
    selected_process_dnd: String,
    popups: Popups,
    parse_error: Option<String>,
    animation: AnimationState,
    transitioning: bool,
}

#[derive(Debug)]
struct Popups {
    local_hack: LocalUI,
    transition: TransitionPopup,
    #[cfg(feature = "scanner")]
    scanner: ScannerPopup,
}

#[derive(Debug)]
struct Communication {
    status_message: Arc<Mutex<String>>,
    in_progress: Arc<std::sync::atomic::AtomicBool>,
    messages: ToastsMessages,
    log_buffer: Arc<Mutex<String>>,
    logger: MyLogger,
}

#[derive(Debug)]
struct AppMeta {
    version: String,
    path: std::path::PathBuf,
    commit: String,
    os_version: String,
    session: String,
}

#[derive(Debug)]
struct TabStates {
    about: AboutTab,
}

struct MyApp {
    app: AppState,
    ui: UIState,
    communication: Communication,
    rpc: Rpc,
    toasts: Toasts,
}

fn default_main_menu_message() -> String {
    format!(
        "Hello {}!\nPlease select a hack from the list.",
        whoami::username()
    )
}

fn get_windows_version() -> Option<String> {
    let hkey = RegKey::predef(HKEY_LOCAL_MACHINE);
    let key = hkey
        .open_subkey("SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion")
        .ok()?;
    let product_name: String = key.get_value("ProductName").ok()?;
    let release_id: String = key.get_value("ReleaseId").ok()?;
    let build: String = key.get_value("CurrentBuild").ok()?;
    Some(format!(
        "{} (Release ID: {}, Build: {})",
        product_name, release_id, build
    ))
}

fn calculate_session(time: String) -> String {
    let session_start = chrono::DateTime::parse_from_rfc3339(&time)
        .unwrap()
        .with_timezone(&chrono::Local);
    let session_duration = chrono::Local::now() - session_start;
    let hours = session_duration.num_hours();
    let minutes = session_duration.num_minutes() % 60;
    let seconds = session_duration.num_seconds() % 60;
    if hours > 0 {
        format!("{} hours and {} minutes", hours, minutes)
    } else if minutes > 0 {
        format!("{} minutes", minutes)
    } else {
        format!("{} seconds", seconds)
    }
}

static LOGGER: OnceLock<MyLogger> = OnceLock::new();

impl MyApp {
    // MARK: Init
    fn new(cc: &eframe::CreationContext) -> Self {
        let mut config = Config::load();
        let config_clone = config.clone();
        let app_path = dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("anarchyloader");

        let logger = MyLogger::init();
        let log_buffer = logger.buffer.clone();
        log::set_max_level(config.log_level.to_level_filter());
        log::info!("Running AnarchyLoader v{}", env!("CARGO_PKG_VERSION"));

        let messages = ToastsMessages::new();
        let mut statistics = Statistics::load();

        statistics.increment_opened_count();

        egui_material_icons::initialize(&cc.egui_ctx);
        cc.egui_ctx.set_theme(config.theme);

        let status_message = Arc::new(Mutex::new(String::new()));
        let in_progress = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let mut parse_error = None;

        let hacks = match hacks::fetch_hacks(
            &config.api_endpoint,
            &config.api_extra_endpoints,
            config.lowercase_hacks,
        ) {
            Ok(hacks) => {
                let mut existing_games: std::collections::HashSet<String> =
                    config.game_order.clone().into_iter().collect();

                for hack in &hacks {
                    if !existing_games.contains(&"CSS".to_string()) && hack.game.starts_with("CSS")
                    {
                        config.game_order.push("CSS".to_string());
                        existing_games.insert("CSS".to_string());
                    } else if !existing_games.contains(&"Rust (NonSteam)".to_string())
                        && hack.game.starts_with("Rust")
                    {
                        config.game_order.push("Rust (NonSteam)".to_string());
                        existing_games.insert("Rust (NonSteam)".to_string());
                    } else if !existing_games.contains(&hack.game)
                        && !hack.game.starts_with("CSS")
                        && !hack.game.starts_with("Rust")
                    {
                        config.game_order.push(hack.game.clone());
                        existing_games.insert(hack.game.clone());
                    }
                    if !config.local_hacks.is_empty()
                        && !config.game_order.contains(&"Added".to_string())
                    {
                        config.game_order.push("Added".to_string());
                    } else if config.local_hacks.is_empty()
                        && config.game_order.contains(&"Added".to_string())
                    {
                        config.game_order.retain(|game| game != "Added");
                    }
                }
                config.save();
                hacks
            }
            Err(err) => {
                log::error!("Failed to fetch hacks: {:?}", err);
                parse_error = Some(err);
                Vec::new()
            }
        };

        if let Err(e) = hacks::save_hacks_to_cache(&hacks) {
            log::error!("Failed to save hacks to cache: {}", e);
        }

        let hacks_processes = get_all_processes(&hacks);

        let account = SteamAccount::new().unwrap_or_else(|_| {
            log::warn!("Failed to get Steam account details");
            SteamAccount::default()
        });

        let rpc = Rpc::new(!config.disable_rpc);
        rpc.update(
            Some(&format!("v{}", env!("CARGO_PKG_VERSION"))),
            Some("Selecting a hack"),
            Some("home"),
        );

        let mut selected_hack = None;

        if config.selected_hack != "" && config.automatically_select_hack {
            selected_hack = get_hack_by_name(&hacks, &config.selected_hack);
            rpc.update(
                None,
                Some(&format!("Selected {}", config.selected_hack)),
                None,
            );
        }

        let mut updater = Updater::default();

        if updater.check_version() {
            log::info!(
                "New version available: {}",
                updater.get_remote_version().unwrap()
            );
        }

        Self {
            app: AppState {
                hacks,
                hacks_processes,
                selected_hack,
                config,
                stats: statistics.clone(),
                account,
                updater,
                cache: CommonMarkCache::default(),
                meta: AppMeta {
                    version: env!("CARGO_PKG_VERSION").to_string(),
                    path: app_path,
                    commit: env!("GIT_HASH").to_string(),
                    os_version: get_windows_version().unwrap_or_else(|| "Unknown".to_string()),
                    session: chrono::Local::now().to_rfc3339(),
                },
            },
            ui: UIState {
                tab: AppTab::default(),
                tab_states: TabStates {
                    about: AboutTab::default(),
                },
                search_query: String::new(),
                main_menu_message: default_main_menu_message(),
                dropped_file: DroppedFile::default(),
                selected_process_dnd: String::new(),
                popups: Popups {
                    local_hack: LocalUI {
                        new_local_dll: String::new(),
                        new_local_process: String::new(),
                        new_local_arch: String::new(),
                    },

                    transition: TransitionPopup {
                        duration: config_clone.transition_duration.clone(),
                        amount: config_clone.transition_amount.clone(),
                    },

                    #[cfg(feature = "scanner")]
                    scanner: ScannerPopup {
                        dll: String::new(),
                        show_results: false,
                    },
                },
                parse_error,
                animation: AnimationState::default(),
                transitioning: false,
            },
            communication: Communication {
                status_message,
                in_progress,
                messages,
                log_buffer,
                logger: logger.clone(),
            },
            rpc,
            toasts: Toasts::default(),
        }
    }

    fn update_rpc_status_selecting(&mut self) {
        let version = format!("v{}", env!("CARGO_PKG_VERSION"));
        let status = if let Some(hack) = &self.app.selected_hack {
            format!("Selected {}", hack.name)
        } else {
            "Selecting hack".to_string()
        };
        self.rpc.update(Some(&version), Some(&status), Some("home"));
    }

    fn render_left_panel(
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

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                            ui.add(
                                egui::TextEdit::singleline(&mut self.ui.search_query)
                                    .hint_text("Search..."),
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
                                ui.label("You enabled 'Show only favorites' and no favorites are available.");
                            } else {
                                ui.label("All games are hidden");

                                if ui.icon_button(ICON_VISIBILITY, "Show all").clicked() {
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
            group_ui.with_layout(egui::Layout::top_down(egui::Align::Min), |layout_ui| {
                layout_ui.with_layout(
                    egui::Layout::top_down_justified(egui::Align::Center),
                    |ui| {
                        ui.heading(game_name);
                    },
                );
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
            ui.with_layout(
                egui::Layout::top_down_justified(egui::Align::Center),
                |ui| {
                    ui.label(RichText::new(version).heading());
                },
            );
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
                })
                .inner;

            self.render_working_state(ui, hack);
            self.render_favorite_button(ui, hack);
            self.render_injection_count(ui, hack);

            if response.clicked() && !in_progress {
                self.select_hack(&hack_clone);
            }

            self.context_menu(&response, ctx, hack);

            response.on_hover_cursor(Clickable);
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

    fn render_working_state(&mut self, ui: &mut egui::Ui, hack: &Hack) {
        if !hack.working {
            ui.label(egui_material_icons::icons::ICON_BLOCK);
        }
    }

    fn render_favorite_button(&mut self, ui: &mut egui::Ui, hack: &Hack) {
        let is_favorite = self.app.config.favorites.contains(&hack.name);
        if is_favorite {
            if ui
                .add(
                    egui::Button::new(egui_material_icons::icons::ICON_STAR)
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

    fn toggle_favorite(&mut self, hack_name: String) {
        if self.app.config.favorites.contains(&hack_name) {
            self.app.config.favorites.remove(&hack_name);
        } else {
            self.app.config.favorites.insert(hack_name);
        }
        self.app.config.save();
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

    fn select_hack(&mut self, hack_clone: &Hack) {
        self.app.selected_hack = Some(hack_clone.clone());
        self.app.config.selected_hack = hack_clone.name.clone();
        self.app.config.save();

        let mut status = self.communication.status_message.lock().unwrap();
        *status = String::new();

        self.rpc
            .update(None, Some(&format!("Selected {}", hack_clone.name)), None);
    }

    fn render_central_panel(&mut self, ctx: &egui::Context, theme_color: egui::Color32) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(selected) = self.app.selected_hack.clone() {
                self.display_hack_details(ui, ctx, &selected, theme_color);
            } else {
                center_vertical(ui, |ui| {
                    ui.label(self.ui.main_menu_message.clone());
                });
            }
        });
    }

    // MARK: Home tab
    fn render_home_tab(&mut self, ctx: &egui::Context, theme_color: egui::Color32) {
        self.handle_key_events(ctx);

        let hacks_by_game = self.group_hacks_by_game();

        self.render_left_panel(ctx, hacks_by_game);
        self.render_central_panel(ctx, theme_color);
    }

    fn render_tabs(&mut self, ctx: &egui::Context, tab: AppTab, theme_color: egui::Color32) {
        match tab {
            AppTab::Home => self.render_home_tab(ctx, theme_color),
            AppTab::Settings => self.render_settings_tab(ctx),
            AppTab::About => self.render_about_tab(ctx),
            AppTab::Logs => self.render_logs_tab(ctx),
            AppTab::Debug => self.render_debug_tab(ctx),
        }
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

        if self.ui.parse_error.is_some() {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(130.0);
                    ui.colored_label(
                        egui::Color32::RED,
                        RichText::new(self.ui.parse_error.as_ref().unwrap())
                            .size(24.0)
                            .strong(),
                    );

                    ui.label("API Endpoint (editable):");
                    if ui
                        .text_edit_singleline(&mut self.app.config.api_endpoint)
                        .changed()
                    {
                        self.app.config.save();
                    }

                    ui.add_space(5.0);

                    if ui.cbutton("Reset config (possible fix)").clicked() {
                        self.app.config = Config::default();
                        self.app.config.save();
                    }

                    ui.add_space(5.0);

                    if ui.cbutton("Exit").clicked() {
                        std::process::exit(0);
                    }
                });
            });
            return;
        }

        if self.app.updater.need_update && !self.app.config.skip_update_check {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(130.0);
                    ui.colored_label(
                        egui::Color32::GREEN,
                        RichText::new("New version available!").size(24.0).strong(),
                    );

                    ui.label(format!(
                        "Newest version is: {} (you are on {})",
                        self.app.updater.new_version.as_ref().unwrap(),
                        self.app.meta.version
                    ));
                    ui.label("Please download the latest version from the website.");

                    ui.add_space(5.0);

                    ui.clink(
                        "Visit Website",
                        "https://github.com/AnarchyLoader/AnarchyLoader/releases/latest",
                    );

                    if ui
                        .ccheckbox(
                            &mut self.app.config.skip_update_check,
                            "Skip update check (you can disable it in settings)",
                        )
                        .changed()
                    {
                        self.app.config.save();
                    };

                    ui.add_space(5.0);

                    if ui.cbutton("Exit").clicked() {
                        std::process::exit(0);
                    }
                });
            });
            return;
        }

        if self.app.stats.opened_count == 1 && self.ui.animation.phase != AnimationPhase::Complete {
            // uncomment to show always show intro screen
            // if true {
            let dt = ctx.input(|i| i.unstable_dt);
            self.update_animation(dt);
            self.render_intro_screen(ctx);

            if self.ui.animation.phase != AnimationPhase::Complete {
                ctx.request_repaint();
            }

            return;
        }

        self.render_top_panel(ctx);

        self.handle_dnd(ctx);
        self.handle_received_messages();

        egui::CentralPanel::default().show(ctx, |ui| {
            if self.app.config.enable_tab_animations {
                let transition_style = TransitionStyle {
                    easing: easing::cubic_out,
                    t_type: TransitionType::HorizontalMove,
                    duration: self.app.config.transition_duration,
                    amount: self.app.config.transition_amount,
                };

                let state = animated_pager(
                    ui,
                    self.ui.tab.clone(),
                    &transition_style,
                    Id::new("tabs"),
                    |_, tab| self.render_tabs(ctx, tab, theme_color),
                );

                self.ui.transitioning = state.animation_running;
            } else {
                self.render_tabs(ctx, self.ui.tab.clone(), theme_color);
                ctx.inspection_ui(ui);
            }
        });
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.rpc.sender.send(RpcUpdate::Shutdown).ok();

        log::info!(
            "Your session was running for: {}",
            calculate_session(self.app.meta.session.clone())
        );
    }
}
