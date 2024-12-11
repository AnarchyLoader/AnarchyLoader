#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod custom_widgets;
mod downloader;
mod hacks;
mod inject;

use std::{
    collections::BTreeMap,
    env, fs,
    path::Path,
    process::Command,
    sync::{
        mpsc::{self, Receiver, Sender, TryRecvError},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

use config::Config;
use custom_widgets::{Button, SelectableLabel};
use eframe::{
    egui::{self, RichText, Spinner},
    App,
};
use egui::{CursorIcon::PointingHand as Clickable, Sense};
use egui_notify::Toasts;
use hacks::Hack;
use is_elevated::is_elevated;

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
    let app = MyApp::new();

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
        Box::new(|_cc| Ok(Box::new(app))),
    )
    .unwrap();
}

#[derive(Debug, Clone, PartialEq)]
enum AppTab {
    Home,
    Settings,
    About,
    Debug
}

impl Default for AppTab {
    fn default() -> Self {
        AppTab::Home
    }
}

struct MyApp {
    hacks: Vec<Hack>,
    selected_hack: Option<Hack>,
    status_message: Arc<Mutex<String>>,
    parse_error: Option<String>,
    app_version: String,
    inject_in_progress: Arc<std::sync::atomic::AtomicBool>,
    tab: AppTab,
    search_query: String,
    main_menu_message: String,
    config: Config,
    toasts: Toasts,
    error_sender: Sender<String>,
    error_receiver: Receiver<String>,
}

impl MyApp {
    // MARK: Init
    fn new() -> Self {
        let (error_sender, error_receiver) = mpsc::channel();
        let config = Config::load_config();
        let status_message = Arc::new(Mutex::new(String::new()));
        let inject_in_progress = Arc::new(std::sync::atomic::AtomicBool::new(false));

        let hacks = match hacks::Hack::fetch_hacks(&config.api_endpoint) {
            Ok(hacks) => hacks,
            Err(err) => {
                return Self {
                    hacks: Vec::new(),
                    selected_hack: None,
                    status_message,
                    parse_error: Some(err),
                    app_version: env!("CARGO_PKG_VERSION").to_string(),
                    inject_in_progress,
                    tab: AppTab::default(),
                    search_query: String::new(),
                    main_menu_message: "Please select a cheat from the list.".to_string(),
                    config: config,
                    toasts: Toasts::default(),
                    error_sender: error_sender,
                    error_receiver: error_receiver,
                }
            }
        };

        Self {
            hacks,
            selected_hack: None,
            status_message,
            parse_error: None,
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            inject_in_progress,
            tab: AppTab::default(),
            search_query: String::new(),
            main_menu_message: "Please select a cheat from the list.".to_string(),
            config: config,
            toasts: Toasts::default(),
            error_sender: error_sender,
            error_receiver: error_receiver,
        }
    }

    // MARK: Top panel
    fn render_top_panel(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.add_space(5.0);
            ui.horizontal(|ui| {
                if ui
                    .cselectable_label(self.tab == AppTab::Home, "Home")
                    .clicked()
                {
                    self.tab = AppTab::Home;
                }
                if ui
                    .cselectable_label(self.tab == AppTab::Settings, "Settings")
                    .clicked()
                {
                    self.tab = AppTab::Settings;
                }
                if ui
                    .cselectable_label(self.tab == AppTab::About, "About")
                    .clicked()
                {
                    self.tab = AppTab::About;
                }
                if ctx.input_mut(|i| i.modifiers.alt) {
                    if ui
                        .cselectable_label(self.tab == AppTab::Debug, "Debug")
                        .clicked()
                    {
                        self.tab = AppTab::Debug;
                    }
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut self.search_query).hint_text("Search..."),
                    );
                });
            });
            ui.add_space(5.0);
        });
    }

    fn reset_config(&mut self) {
        self.config = Config::default();
        self.config.save_config();
    }

    // MARK: Home tab
    fn render_home_tab(&mut self, ctx: &egui::Context, theme_color: egui::Color32) {
        match self.error_receiver.try_recv() {
            Ok(error) => {
                if error.starts_with("SUCCESS: ") {
                    let name = error.trim_start_matches("SUCCESS: ").to_string();
                    self.toasts
                        .success(format!("Successfully injected {}", name))
                        .duration(Some(Duration::from_secs(4)));
                } else {
                    self.toasts
                        .error(error)
                        .duration(Some(Duration::from_secs(7)));
                }
            }
            Err(TryRecvError::Empty) => {}
            Err(e) => {
                eprintln!("Error receiving from channel: {:?}", e);
            }
        }

        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
            self.selected_hack = None;
        }

        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter)) {
            if let Some(selected) = &self.selected_hack {
                if selected.game == "CSGO" {
                    self.manual_map_injection(
                        selected.clone(),
                        ctx.clone(),
                        self.error_sender.clone(),
                    );
                } else {
                    self.start_injection(selected.clone(), ctx.clone(), self.error_sender.clone());
                }
            }
        }

        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::F5)) {
            self.main_menu_message = "Fetching hacks...".to_string();
            ctx.request_repaint();
            self.hacks = match hacks::Hack::fetch_hacks(&self.config.api_endpoint) {
                Ok(hacks) => {
                    self.main_menu_message = "Please select a cheat from the list.".to_string();
                    ctx.request_repaint();
                    hacks
                }
                Err(_err) => {
                    self.main_menu_message = "Failed to fetch hacks.".to_string();
                    Vec::new()
                }
            };

            self.toasts.info("Hacks refreshed.");
        }

        let mut hacks_by_game: BTreeMap<String, BTreeMap<String, Vec<Hack>>> = BTreeMap::new();

        for hack in self.hacks.clone() {
            if self.config.show_only_favorites && !self.config.favorites.contains(&hack.name) {
                continue;
            }

            if self.search_query.is_empty()
                || hack
                    .name
                    .to_lowercase()
                    .contains(&self.search_query.to_lowercase())
            {
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
        }

        hacks_by_game.retain(|_, versions| {
            versions.retain(|_, hacks| !hacks.is_empty());
            !versions.is_empty()
        });

        // MARK: Left panel
        egui::SidePanel::left("left_panel")
            .resizable(true)
            .default_width(200.0)
            .max_width(300.0)
            .show(ctx, |ui| {
                ui.add_space(5.0);

                egui::ScrollArea::vertical().show(ui, |ui| {
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
                                            egui::Layout::top_down_justified(egui::Align::Center),
                                            |ui| ui.label(RichText::new(version).heading()),
                                        );
                                    }

                                    for hack in hacks {
                                        let hack_clone = hack.clone();
                                        ui.horizontal(|ui| {
                                            let is_favorite =
                                                self.config.favorites.contains(&hack.name);
                                            let label = if is_favorite {
                                                RichText::new(&hack.name)
                                                    .color(self.config.favorites_color)
                                            } else {
                                                RichText::new(&hack.name)
                                            };

                                            let response = ui.selectable_label(
                                                self.selected_hack.as_ref() == Some(&hack),
                                                label,
                                            );

                                            let hovered = response.hovered();

                                            if hovered || is_favorite {
                                                let favorite_icon =
                                                    if is_favorite { "★" } else { "☆" };
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
                                                        self.config.favorites.remove(&hack.name);
                                                    } else {
                                                        self.config
                                                            .favorites
                                                            .insert(hack.name.clone());
                                                    }
                                                    self.config.save_config();
                                                }
                                            }

                                            let file_path_owned = hack.file_path.clone();
                                            let ctx_clone = ctx.clone();
                                            let status_message = Arc::clone(&self.status_message);

                                            if response.clicked() {
                                                self.selected_hack = Some(hack_clone.clone());
                                                let mut status =
                                                    self.status_message.lock().unwrap();
                                                *status = String::new();
                                            }

                                            // MARK: Context menu
                                            response.context_menu(|ui| {
                                                if is_favorite {
                                                    if ui.cbutton("Remove from favorites").clicked()
                                                    {
                                                        self.config.favorites.remove(&hack.name);
                                                        self.config.save_config();
                                                        self.toasts.info(format!("Removed {} from favorites.", hack.name));
                                                        ui.close_menu();
                                                    }
                                                } else {
                                                    if ui.cbutton("Add to favorites").clicked()
                                                    {
                                                        self.config.favorites.insert(hack.name.clone());
                                                        self.config.save_config();
                                                        self.toasts.info(format!("Added {} to favorites.", hack.name));
                                                        ui.close_menu();
                                                    }
                                                }

                                                if Path::new(&file_path_owned).exists() {
                                                    if ui.button_with_tooltip("Open in Explorer", "Open the file location in Explorer").clicked() {
                                                        if let Err(e) = Command::new("explorer.exe")
                                                            .arg(format!(
                                                                "/select,{}",
                                                                hack.file_path.to_string_lossy()
                                                            ))
                                                            .spawn()
                                                        {
                                                            let mut status =
                                                                self.status_message.lock().unwrap();
                                                            *status = format!(
                                                                "Failed to open Explorer: {}",
                                                                e
                                                            );
                                                            self.toasts.error(format!("Failed to open Explorer: {}", e));
                                                        }
                                                    }
                                                }

                                                if ui.button_with_tooltip("Uninstall", "Uninstall the selected hack").clicked() {
                                                        if let Err(e) = std::fs::remove_file(&file_path_owned) {
                                                            let mut status =
                                                                self.status_message.lock().unwrap();
                                                            *status = format!(
                                                                "Failed to uninstall: {}",
                                                                e
                                                            );
                                                        } else {
                                                            let mut status =
                                                                self.status_message.lock().unwrap();
                                                            *status = "Uninstall successful.".to_string();
                                                        }
                                                    }

                                                if ui.button_with_tooltip("Reinstall", "Reinstall the selected hack").clicked()
                                                {
                                                    thread::spawn(move || {
                                                        if !Path::new(&file_path_owned).exists() {
                                                            let mut status =
                                                                status_message.lock().unwrap();
                                                            *status =
                                                                "Failed to reinstall: file does not exist.".to_string();
                                                            ctx_clone.request_repaint();
                                                            return;
                                                        }

                                                        {
                                                            let mut status =
                                                                status_message.lock().unwrap();
                                                            *status = "Reinstalling...".to_string();
                                                            ctx_clone.request_repaint();
                                                        }

                                                        if let Err(e) =
                                                            fs::remove_file(&file_path_owned)
                                                        {
                                                            let mut status =
                                                                status_message.lock().unwrap();
                                                            *status = format!(
                                                                "Failed to delete file: {}",
                                                                e
                                                            );
                                                            ctx_clone.request_repaint();
                                                            return;
                                                        }

                                                        match hack.download(
                                                            file_path_owned
                                                                .to_string_lossy()
                                                                .to_string(),
                                                        ) {
                                                            Ok(_) => {
                                                                let mut status =
                                                                    status_message.lock().unwrap();
                                                                *status =
                                                                    "Reinstalled.".to_string();
                                                                ctx_clone.request_repaint();
                                                            }
                                                            Err(e) => {
                                                                let mut status =
                                                                    status_message.lock().unwrap();
                                                                *status = format!(
                                                                    "Failed to reinstall: {}",
                                                                    e
                                                                );
                                                                ctx_clone.request_repaint();
                                                            }
                                                        }
                                                    });
                                                }
                                            });
                                            response.on_hover_cursor(Clickable);
                                        });
                                    }
                                }
                            });
                        });
                        ui.add_space(10.0);
                    }
                });
            });

        // MARK: Selected hack panel
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(10.0);
            if let Some(selected) = &self.selected_hack {
                let is_csgo = selected.game == "CSGO";

                ui.horizontal(|ui| {
                    ui.heading(&selected.name);
                    ui.label(RichText::new(format!("by {}", selected.author)).color(theme_color));
                    ui.hyperlink_to("(source)", &selected.source)
                        .on_hover_text(&selected.source)
                });
                ui.separator();
                ui.label(&selected.description);

                ui.add_space(5.0);
                // MARK: Inject button
                if ui.button(format!("Inject {}", selected.name))
                    .on_hover_cursor(Clickable)
                    .on_hover_text(&selected.file)
                    .clicked()
                {
                    self.toasts
                        .custom(format!("Injecting {}", selected.name), "⌛".to_string(), egui::Color32::from_rgb(150, 200, 210))
                        .duration(Some(Duration::from_secs(2)));
                    if is_csgo {
                        self.manual_map_injection(selected.clone(), ctx.clone(), self.error_sender.clone());
                    } else {
                        self.start_injection(selected.clone(), ctx.clone(), self.error_sender.clone());
                    }
                }
                if !is_elevated() && is_csgo && !self.config.hide_csgo_warning {
                    ui.label(
                        RichText::new("If you encounter an error stating that csgo.exe is not found try running the loader as an administrator\nYou can disable this warning in the settings.")
                            .size(11.0)
                            .color(egui::Color32::YELLOW),
                    );
                }

                let inject_in_progress = self
                    .inject_in_progress
                    .load(std::sync::atomic::Ordering::SeqCst);

                if inject_in_progress {
                    ui.add_space(10.0);
                    let status = self.status_message.lock().unwrap().clone();
                    ui.horizontal(|ui| {
                        ui.add(Spinner::new());
                        ui.add_space(5.0);
                        ui.label(
                            RichText::new(&status).color(if status.starts_with("Failed") {
                                egui::Color32::RED
                            } else {
                                theme_color
                            }),
                        );
                        ctx.request_repaint();
                    });
                } else {
                    ui.add_space(10.0);
                    let status = self.status_message.lock().unwrap().clone();
                    if !status.is_empty() {
                        let color = if status.starts_with("Failed") {
                            egui::Color32::RED
                        } else {
                            theme_color
                        };
                        ui.label(RichText::new(&status).color(color));
                    }
                }
            } else {
                ui.vertical_centered(|ui| {
                    ui.add_space(150.0);
                    ui.label(self.main_menu_message.clone());
                });
            }
        });

        self.toasts.show(ctx);
    }

    // MARK: Settings Tab
    fn render_settings_tab(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading("Settings");
                ui.separator();

                if ui
                    .checkbox(
                        &mut self.config.show_only_favorites,
                        "Show only favorite hacks",
                    )
                    .on_hover_cursor(Clickable)
                    .changed()
                {
                    self.config.save_config();
                }

                ui.add_space(10.0);

                if ui
                    .checkbox(
                        &mut self.config.skip_injects_delay,
                        "Skip injects delay (visual)",
                    )
                    .on_hover_cursor(Clickable)
                    .changed()
                {
                    self.config.save_config();
                }

                ui.add_space(10.0);

                if ui
                    .checkbox(&mut self.config.hide_csgo_warning, "Hide CSGO warning")
                    .on_hover_cursor(Clickable)
                    .changed()
                {
                    self.config.save_config();
                }

                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    ui.label("Favorites Color:");
                    if ui
                        .color_edit_button_srgba(&mut self.config.favorites_color)
                        .on_hover_cursor(Clickable)
                        .changed()
                    {
                        self.config.save_config();
                    }
                });

                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    ui.label("API Endpoint:");
                    if ui
                        .text_edit_singleline(&mut self.config.api_endpoint)
                        .changed()
                    {
                        self.config.save_config();
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("CDN Endpoint:");
                    if ui
                        .text_edit_singleline(&mut self.config.cdn_endpoint)
                        .changed()
                    {
                        self.config.save_config();
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("CSGO Injector:");
                    if ui
                        .text_edit_singleline(&mut self.config.csgo_injector)
                        .changed()
                    {
                        self.config.save_config();
                    }
                });

                ui.add_space(10.0);

                if ui.cbutton("Reset settings").clicked() {
                    self.reset_config();
                    self.toasts.success("Settings reset.");
                }

                if ui.cbutton("Open loader folder").clicked() {
                    let downloads_dir = dirs::config_dir()
                        .unwrap_or_else(|| std::path::PathBuf::from("."))
                        .join("anarchyloader");
                    let _ = opener::open(downloads_dir);
                }
            });
        });
        self.toasts.show(ctx);
    }

    // MARK: About Tab
    fn render_about_tab(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading("About");
                ui.separator();
                if ui
                    .add(
                        egui::Image::new(egui::include_image!("../resources/img/icon.ico"))
                            .max_width(100.0)
                            .rounding(10.0)
                            .sense(Sense::click()),
                    )
                    .clicked()
                {
                    self.toasts.info("Hello there!");
                }
                ui.label(RichText::new(format!("v{}", self.app_version)).size(15.0));
                ui.add_space(10.0);
                ui.label(
                    RichText::new(
                        "AnarchyLoader is a free and open-source cheat loader for various games.",
                    )
                    .size(16.0),
                );
                ui.add_space(5.0);
                ui.hyperlink_to("by dest4590", "https://github.com/dest4590")
                    .on_hover_text("https://github.com/dest4590");
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if ui.cbutton("Visit Website").clicked() {
                        let _ = opener::open("https://anarchy.my");
                    }
                    if ui.cbutton("Github Repository").clicked() {
                        let _ = opener::open("https://github.com/AnarchyLoader/AnarchyLoader");
                    }
                });

                ui.add_space(5.0);
                ui.label("Keybinds:");
                ui.label("F5 - Refresh hacks");
                ui.label("Enter - Inject selected hack");
                ui.label("Escape - Deselect hack");
                ui.label("Hold Alt - Debug tab");
            });
        });
        self.toasts.show(ctx);
    }

    fn render_debug_tab(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.add_space(10.0);
                ui.heading("Debug");
                ui.separator();

                let debug_info = vec![
                    ("Config:", format!("{:#?}", self.config)),
                    ("Hacks:", format!("{:#?}", self.hacks)),
                    ("Selected Hack:", format!("{:#?}", self.selected_hack)),
                    ("Status Message:", format!("{:#?}", self.status_message)),
                    ("Parse Error:", format!("{:#?}", self.parse_error)),
                    ("Inject in Progress:", format!("{:#?}", self.inject_in_progress)),
                    ("Search Query:", format!("{:#?}", self.search_query)),
                    ("Main Menu Message:", format!("{:#?}", self.main_menu_message)),
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

                if ui.cbutton("Copy debug info").on_hover_cursor(Clickable).clicked() {
                    let debug_info = debug_info
                        .iter()
                        .map(|(label, value)| format!("{}: {}\n", label, value))
                        .collect::<String>();
                    ui.output_mut(|o| o.copied_text = debug_info);
                    self.toasts.success("Debug info copied to clipboard.");
                }
            });
        });
        self.toasts.show(ctx);
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
                        self.config.save_config();
                    }
                });
            });
            return;
        }

        self.render_top_panel(ctx);

        match self.tab {
            AppTab::Home => self.render_home_tab(ctx, theme_color),
            AppTab::Settings => self.render_settings_tab(ctx),
            AppTab::About => self.render_about_tab(ctx),
            AppTab::Debug => self.render_debug_tab(ctx),
        }
    }
}
