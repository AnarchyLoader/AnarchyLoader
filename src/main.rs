#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
// ^-- disable console window in release mode

mod games;
mod inject;
mod scanner;
mod tabs;
mod utils;

use std::{
    env,
    sync::{Arc, Mutex, OnceLock},
};

use eframe::{
    egui::{self, RichText},
    App,
};
use egui::{emath::easing, include_image, DroppedFile, FontFamily, FontId, Id, Image, Vec2};
use egui_alignments::center_vertical;
use egui_commonmark::CommonMarkCache;
use egui_notify::Toasts;
use egui_text_animation::{AnimationType, TextAnimator};
use egui_transition_animation::{animated_pager, TransitionStyle, TransitionType};
use games::local::LocalUI;
use is_elevated::is_elevated;
#[cfg(feature = "scanner")]
use scanner::scanner::ScannerPopup;
use tabs::top_panel::AppTab;
use utils::{
    api::{
        hacks,
        hacks::{get_hack_by_name, Hack},
        updater::Updater,
    },
    config::Config,
    logger::MyLogger,
    rpc::{Rpc, RpcUpdate},
    stats::Statistics,
    steam::SteamAccount,
    ui::{
        messages::ToastsMessages,
        native_theme,
        widgets::{Button, CheckBox, Hyperlink},
    },
};

use crate::{
    tabs::{about::AboutTab, home::HomeTab, top_panel::TopPanel},
    utils::{
        helpers::get_windows_version,
        stats::{calculate_session, get_time_difference_in_seconds},
        ui::intro::{AnimationPhase, AnimationState},
    },
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
            .with_min_inner_size(egui::vec2(600.0, 300.0))
            .with_inner_size(egui::vec2(800.0, 400.0))
            .with_icon(Arc::new(load_icon())),
        ..Default::default()
    };

    let title = if !is_elevated() {
        format!("AnarchyLoader v{}", env!("CARGO_PKG_VERSION"))
    } else {
        format!(
            "AnarchyLoader v{} (Administrator)",
            env!("CARGO_PKG_VERSION")
        )
            .to_string()
    };

    eframe::run_native(
        &title,
        native_options,
        Box::new(|cc| Ok(Box::new(MyApp::new(cc)))),
    )
    .unwrap();
}

#[derive(Debug)]
struct AppState {
    hacks: Vec<Hack>,
    selected_hack: Option<Hack>,
    config: Config,
    stats: Statistics,
    updater: Updater,
    meta: AppMeta,
}

#[derive(Debug)]
struct UIState {
    tab: AppTab,
    tabs: TabStates,
    text_animator: TextAnimator,
    mark_cache: CommonMarkCache,
    search_query: String,
    main_menu_message: String,
    dropped_file: DroppedFile,
    selected_process_dnd: String,
    using_cache: bool,
    popups: Popups,
    parse_error: Option<String>,
    animation: AnimationState,
    transitioning: bool,
}

#[derive(Debug)]
struct Popups {
    local_hack: LocalUI,
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
    steam_account: SteamAccount,
}

#[derive(Debug)]
struct TabStates {
    top_panel: TopPanel,
    about: AboutTab,
    home: HomeTab,
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

static LOGGER: OnceLock<MyLogger> = OnceLock::new();
impl MyApp {
    // MARK: Init
    fn new(cc: &eframe::CreationContext) -> Self {
        let mut config = Config::load();
        let app_path = dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("anarchyloader");

        let logger = MyLogger::init();
        let log_buffer = logger.buffer.clone();
        log::set_max_level(config.log_level.to_level_filter());
        log::info!(
            "<MAIN> Running AnarchyLoader v{}",
            env!("CARGO_PKG_VERSION")
        );

        let messages = ToastsMessages::new();
        let mut statistics = Statistics::load();
        log::debug!("<MAIN> Statistics loaded: {:?}", statistics);

        statistics.increment_opened_count();
        log::debug!(
            "<MAIN> Application opened count incremented to: {}",
            statistics.opened_count
        );

        egui_material_icons::initialize(&cc.egui_ctx);
        cc.egui_ctx.set_theme(config.display.theme);
        log::debug!("<MAIN> Theme set to: {:?}", config.display.theme);

        let status_message = Arc::new(Mutex::new(String::new()));
        let mut parse_error = None;
        let mut using_cache = false;

        log::info!(
            "<HACKS> Fetching hacks from API endpoint: {}",
            config.api.api_endpoint
        );

        let hacks = match hacks::fetch_hacks(
            &config.api.api_endpoint,
            &config.api.api_extra_endpoints,
            config.lowercase_hacks,
        ) {
            Ok((hacks, cached)) => {
                using_cache = cached;

                let mut existing_games: std::collections::HashSet<String> =
                    config.game_order.clone().into_iter().collect();

                for hack in &hacks {
                    let game_name = if hack.game.starts_with("CSS") {
                        "CSS".to_string()
                    } else if hack.game.starts_with("Rust") {
                        "Rust (NonSteam)".to_string()
                    } else {
                        hack.game.clone()
                    };

                    if !existing_games.contains(&game_name) {
                        config.game_order.push(game_name.clone());
                        existing_games.insert(game_name.clone());
                        log::info!("<MAIN> Added new game to game_order: {}", game_name);
                    }
                }
                if !config.local_hacks.is_empty() && !existing_games.contains("Added") {
                    config.game_order.push("Added".to_string());
                    log::info!(
                        "<MAIN> Added 'Added' category to game_order because local hacks are present"
                    );
                } else if config.local_hacks.is_empty() && existing_games.contains("Added") {
                    config.game_order.retain(|game| game != "Added");
                    log::info!("<MAIN> Removed 'Added' category from game_order because no local hacks are present");
                }
                config.save();
                hacks
            }
            Err(err) => {
                log::error!("<HACKS> Failed to fetch hacks: {:?}", err);
                parse_error = Some(err);
                Vec::new()
            }
        };

        if let Err(e) = hacks::save_hacks_to_cache(&hacks) {
            log::error!("<HACKS> Failed to save hacks to cache: {}", e);
        } else {
            log::info!("<HACKS> Hacks saved to cache successfully.");
        }

        let steam_account = SteamAccount::new().unwrap_or_else(|_| {
            log::warn!("<STEAM> Failed to get Steam account details");
            SteamAccount::default()
        });

        let rpc = Rpc::new(!config.disable_rpc);
        if !config.disable_rpc {
            log::info!("<RPC> Discord RPC initialized");
        }

        rpc.update(
            Some(&format!("v{}", env!("CARGO_PKG_VERSION"))),
            Some("Selecting a hack"),
            Some("home"),
        );

        let mut selected_hack = None;

        if !config.display.selected_hack.is_empty() && config.automatically_select_hack {
            selected_hack = get_hack_by_name(
                &Self::get_all_hacks(&hacks, &config),
                &config.display.selected_hack,
            );
            if selected_hack.is_some() {
                rpc.update(
                    None,
                    Some(&format!("Selected {}", config.display.selected_hack)),
                    None,
                );
            } else {
                log::warn!(
                    "<MAIN> Failed to automatically select hack '{}', hack not found.",
                    config.display.selected_hack
                );
            }
        }

        let mut updater = Updater::default();

        match updater.check_version() {
            Ok(true) => {}
            Ok(false) => {
                log::info!("<UPDATER> No update needed");
            }
            Err(e) => {
                log::error!("<UPDATER> Failed to check for updates: {}", e);
            }
        }

        native_theme::register(&cc.egui_ctx);

        Self {
            app: AppState {
                hacks,
                selected_hack: selected_hack.clone(),
                config: config.clone(),
                stats: statistics.clone(),
                updater,
                meta: AppMeta {
                    version: env!("CARGO_PKG_VERSION").to_string(),
                    path: app_path,
                    commit: env!("GIT_HASH").to_string(),
                    os_version: get_windows_version().unwrap_or_else(|| "Unknown".to_string()),
                    session: chrono::Local::now().to_rfc3339(),
                    steam_account,
                },
            },
            ui: UIState {
                tab: AppTab::default(),
                tabs: TabStates {
                    top_panel: TopPanel::default(),
                    about: AboutTab::default(),
                    home: HomeTab::default(),
                },
                text_animator: TextAnimator::new(
                    &selected_hack.unwrap_or_default().name,
                    FontId::new(19.0, FontFamily::Proportional),
                    if cc.egui_ctx.style().visuals.dark_mode {
                        egui::Color32::LIGHT_GRAY
                    } else {
                        egui::Color32::DARK_GRAY
                    },
                    config.animations.text_speed,
                    AnimationType::FadeIn,
                ),
                mark_cache: CommonMarkCache::default(),
                search_query: String::new(),
                main_menu_message: default_main_menu_message(),
                dropped_file: DroppedFile::default(),
                selected_process_dnd: String::new(),
                using_cache,
                popups: Popups {
                    local_hack: LocalUI {
                        new_local_dll: String::new(),
                        new_local_process: String::new(),
                        new_local_arch: String::new(),
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
                in_progress: Arc::new(std::sync::atomic::AtomicBool::new(false)),
                messages,
                log_buffer,
                logger: logger.clone(),
            },
            rpc,
            toasts: Toasts::default(),
        }
    }

    fn render_central_panel(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(selected) = self.app.selected_hack.clone() {
                self.display_hack_details(ui, ctx, &selected);
            } else {
                center_vertical(ui, |ui| {
                    ui.label(self.ui.main_menu_message.clone());
                    ui.add(
                        Image::new(include_image!("../resources/img/icon.png"))
                            .fit_to_exact_size(Vec2::new(64.0, 64.0)),
                    );
                });
            }
        });
    }

    fn render_tabs(&mut self, ctx: &egui::Context, tab: AppTab) {
        match tab {
            AppTab::Home => self.render_home_tab(ctx),
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

        if !self.app.config.display.disable_hack_name_animation {
            self.setup_text_animator_color(ctx);
        }

        if self.ui.parse_error.is_some() {
            log::error!(
                "<MAIN> API Parse Error detected, showing error screen to user: {:?}",
                self.ui.parse_error
            );

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
                        .text_edit_singleline(&mut self.app.config.api.api_endpoint)
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

        if self.app.updater.need_update && !self.app.config.display.skip_update_check {
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
                            &mut self.app.config.display.skip_update_check,
                            "Skip update check",
                        )
                        .changed()
                    {
                        self.toasts
                            .info("You can enable update checks in settings.");
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
            if self.app.config.animations.tab_animations {
                let transition_style = TransitionStyle {
                    easing: easing::cubic_out,
                    t_type: TransitionType::HorizontalMove,
                    duration: self.app.config.animations.duration,
                    amount: self.app.config.animations.amount,
                };

                let state = animated_pager(
                    ui,
                    self.ui.tab.clone(),
                    &transition_style,
                    Id::new("tabs"),
                    |_, tab| self.render_tabs(ctx, tab),
                );

                self.ui.transitioning = state.animation_running;
            } else {
                self.render_tabs(ctx, self.ui.tab.clone());
            }
        });
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.rpc.sender.send(RpcUpdate::Shutdown).ok();
        log::info!("<MAIN> Sent shutdown signal to Discord RPC");

        log::info!(
            "<MAIN> Your session was running for: {}",
            calculate_session(self.app.meta.session.clone())
        );

        self.app
            .stats
            .increment_total_time(get_time_difference_in_seconds(
                self.app.meta.session.parse().unwrap(),
            ));
        log::info!("<MAIN> Total time incremented and session statistics updated.");

        self.app.stats.save();
        log::info!("<MAIN> Statistics saved on application exit.");
        log::info!("<MAIN> Application exited gracefully.");
    }
}
