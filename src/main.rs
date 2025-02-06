#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod games;
mod hacks;
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
use egui::{emath::easing, include_image, DroppedFile, Id, Image, Vec2};
use egui_alignments::center_vertical;
use egui_commonmark::CommonMarkCache;
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
    stats::Statistics,
    steam::SteamAccount,
    updater::Updater,
};
use winreg::{enums::HKEY_LOCAL_MACHINE, RegKey};

use crate::{
    tabs::about::AboutTab,
    utils::{
        intro::{AnimationPhase, AnimationState},
        stats::{calculate_session, get_time_difference_in_seconds},
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
            .with_min_inner_size(egui::vec2(600.0, 200.0))
            .with_inner_size(egui::vec2(800.0, 400.0))
            .with_icon(Arc::new(load_icon())),
        ..Default::default()
    };
    eframe::run_native(
        if !is_elevated() {
            log::info!("[MAIN] Application started as normal user");
            "AnarchyLoader"
        } else {
            log::info!("[MAIN] Application started as administrator");
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
        log::info!(
            "[MAIN] Running AnarchyLoader v{}",
            env!("CARGO_PKG_VERSION")
        );

        let messages = ToastsMessages::new();
        let mut statistics = Statistics::load();
        log::debug!("[MAIN] Statistics loaded: {:?}", statistics);

        statistics.increment_opened_count();
        log::debug!(
            "[MAIN] Application opened count incremented to: {}",
            statistics.opened_count
        );

        egui_material_icons::initialize(&cc.egui_ctx);
        cc.egui_ctx.set_theme(config.theme);
        log::debug!("[MAIN] Theme set to: {:?}", config.theme);

        let status_message = Arc::new(Mutex::new(String::new()));
        let in_progress = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let mut parse_error = None;

        log::info!(
            "[MAIN] Fetching hacks from API endpoint: {}",
            config.api_endpoint
        );
        let hacks = match hacks::fetch_hacks(
            &config.api_endpoint,
            &config.api_extra_endpoints,
            config.lowercase_hacks,
        ) {
            Ok(hacks) => {
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
                        log::info!("[MAIN] Added new game to game_order: {}", game_name);
                    }
                }
                if !config.local_hacks.is_empty() && !existing_games.contains(&"Added".to_string())
                {
                    config.game_order.push("Added".to_string());
                    log::info!(
                        "[MAIN] Added 'Added' category to game_order because local hacks are present"
                    );
                } else if config.local_hacks.is_empty()
                    && existing_games.contains(&"Added".to_string())
                {
                    config.game_order.retain(|game| game != "Added");
                    log::info!("[MAIN] Removed 'Added' category from game_order because no local hacks are present");
                }
                config.save();
                log::info!("[MAIN] Configuration saved after updating game_order");
                hacks
            }
            Err(err) => {
                log::error!("[MAIN] Failed to fetch hacks: {:?}", err);
                parse_error = Some(err);
                Vec::new()
            }
        };

        if let Err(e) = hacks::save_hacks_to_cache(&hacks) {
            log::error!("[MAIN] Failed to save hacks to cache: {}", e);
        } else {
            log::info!("[MAIN] Hacks saved to cache successfully.");
        }

        let hacks_processes = get_all_processes(&hacks);

        let account = SteamAccount::new().unwrap_or_else(|_| {
            log::warn!("[MAIN] Failed to get Steam account details");
            SteamAccount::default()
        });
        log::info!(
            "[MAIN] Steam Account details: {:?}",
            account.get_censoured()
        );

        let rpc = Rpc::new(!config.disable_rpc);
        if !config.disable_rpc {
            log::info!("[MAIN] Discord RPC initialized");
        } else {
            log::info!("[MAIN] Discord RPC disabled in config");
        }
        rpc.update(
            Some(&format!("v{}", env!("CARGO_PKG_VERSION"))),
            Some("Selecting a hack"),
            Some("home"),
        );

        let mut selected_hack = None;

        if config.selected_hack != "" && config.automatically_select_hack {
            selected_hack = get_hack_by_name(&hacks, &config.selected_hack);
            if selected_hack.is_some() {
                log::info!(
                    "[MAIN] Automatically selected hack from config: {}",
                    config.selected_hack
                );
                rpc.update(
                    None,
                    Some(&format!("Selected {}", config.selected_hack)),
                    None,
                );
            } else {
                log::warn!(
                    "[MAIN] Failed to automatically select hack '{}', hack not found.",
                    config.selected_hack
                );
            }
        }

        let mut updater = Updater::default();

        log::info!("[MAIN] Checking for updates...");
        match updater.check_version() {
            Ok(true) => {
                log::info!(
                    "[MAIN] Update needed, new version: {}",
                    updater.new_version.as_ref().unwrap()
                );
            }
            Ok(false) => {
                log::info!("[MAIN] No update needed");
            }
            Err(e) => {
                log::error!("[MAIN] Failed to check for updates: {}", e);
            }
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
        log::debug!(
            "[MAIN] Updating RPC status to: version={}, status={}",
            version,
            status
        );
        self.rpc.update(Some(&version), Some(&status), Some("home"));
    }

    fn render_central_panel(&mut self, ctx: &egui::Context, highlight_color: egui::Color32) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(selected) = self.app.selected_hack.clone() {
                self.display_hack_details(ui, ctx, &selected, highlight_color);
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

    fn render_tabs(&mut self, ctx: &egui::Context, tab: AppTab, highlight_color: egui::Color32) {
        match tab {
            AppTab::Home => self.render_home_tab(ctx, highlight_color),
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
        let highlight_color = if is_dark_mode {
            egui::Color32::LIGHT_GRAY
        } else {
            egui::Color32::DARK_GRAY
        };

        if self.ui.parse_error.is_some() {
            log::error!(
                "[MAIN] API Parse Error detected, showing error screen to user: {:?}",
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
                        .text_edit_singleline(&mut self.app.config.api_endpoint)
                        .changed()
                    {
                        log::info!(
                            "[MAIN] API Endpoint changed by user to: {}",
                            self.app.config.api_endpoint
                        );
                        self.app.config.save();
                        log::info!("[MAIN] Configuration saved after API endpoint change.");
                    }

                    ui.add_space(5.0);

                    if ui.cbutton("Reset config (possible fix)").clicked() {
                        log::info!("[MAIN] User clicked 'Reset config'");
                        self.app.config = Config::default();
                        self.app.config.save();
                        log::info!("[MAIN] Default configuration saved after reset.");
                    }

                    ui.add_space(5.0);

                    if ui.cbutton("Exit").clicked() {
                        log::info!("[MAIN] User clicked 'Exit' due to API parse error.");
                        std::process::exit(0);
                    }
                });
            });
            return;
        }

        if self.app.updater.need_update && !self.app.config.skip_update_check {
            log::info!(
                "[MAIN] Update available and update check is not skipped, showing update screen."
            );
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
                        log::info!(
                            "[MAIN] 'Skip update check' option changed to: {}",
                            self.app.config.skip_update_check
                        );
                        self.app.config.save();
                        log::info!("[MAIN] Configuration saved after 'skip update check' change.");
                    };

                    ui.add_space(5.0);

                    if ui.cbutton("Exit").clicked() {
                        log::info!("[MAIN] User clicked 'Exit' due to update available screen.");
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
                    |_, tab| self.render_tabs(ctx, tab, highlight_color),
                );

                self.ui.transitioning = state.animation_running;
            } else {
                self.render_tabs(ctx, self.ui.tab.clone(), highlight_color);
                ctx.inspection_ui(ui);
            }
        });
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.rpc.sender.send(RpcUpdate::Shutdown).ok();
        log::info!("[MAIN] Sent shutdown signal to Discord RPC");

        log::info!(
            "[MAIN] Your session was running for: {}",
            calculate_session(self.app.meta.session.clone())
        );

        self.app
            .stats
            .increment_total_time(get_time_difference_in_seconds(
                self.app.meta.session.parse().unwrap(),
            ));
        log::info!("[MAIN] Total time incremented and session statistics updated.");

        self.app.stats.save();
        log::info!("[MAIN] Statistics saved on application exit.");
        log::info!("[MAIN] Application exited gracefully.");
    }
}
