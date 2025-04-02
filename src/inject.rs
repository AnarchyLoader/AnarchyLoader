use std::{
    io::{BufRead, BufReader},
    path::PathBuf,
    process::{Command, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::Sender,
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

use eframe::egui::{self};
use egui::ViewportCommand;
use proc_mem::Process;
use sysinfo::System;

use crate::{
    utils::{
        api::downloader::download_file,
        helpers::{is_process_running, start_cs_prompt},
        ui::messages::MessageSender,
    },
    Hack, MyApp,
};

const STEAM_EXE: &str = "steam.exe";
const CLIENT_DLL: &str = "client.dll";

pub(crate) fn change_status_message(status_message: &Arc<Mutex<String>>, message: &str) {
    let mut status = status_message.lock().unwrap();
    *status = message.to_string();
}

impl MyApp {
    pub fn delete_injectors(&mut self, arch: &str) -> Result<(), String> {
        let injectors = match arch {
            "both" => vec!["AnarchyInjector_x86.exe", "AnarchyInjector_x64.exe"],
            "x86" => vec!["AnarchyInjector_x86.exe"],
            "x64" => vec!["AnarchyInjector_x64.exe"],
            _ => return Err("Invalid architecture specified".to_string()),
        };

        let mut errors = Vec::new();
        for injector in &injectors {
            let injector_path = self.get_injector_path(injector);
            if injector_path.exists() {
                if let Err(e) = std::fs::remove_file(&injector_path) {
                    log::error!("<INJECTION> Failed to delete {} injector: {}", injector, e);
                    errors.push(format!("Failed to delete {} injector: {}", injector, e));
                } else {
                    log::info!("<INJECTION> Deleted {}", injector);
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors.join("\n"))
        }
    }

    fn get_injector_path(&self, injector_name: &str) -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("anarchyloader")
            .join(injector_name)
    }

    pub fn download_injectors(&mut self, message_sender: Sender<String>, nightly: bool) {
        let message_sender_clone = message_sender.clone();

        thread::spawn(move || {
            if nightly {
                Self::download_nightly_injectors(message_sender_clone);
            } else {
                Self::download_stable_injectors(message_sender_clone);
            }
        });
    }

    fn download_stable_injectors(message_sender: Sender<String>) {
        let injectors = vec!["AnarchyInjector_x86.exe", "AnarchyInjector_x64.exe"];
        for injector in injectors {
            match download_file(injector, None) {
                Ok(_) => {
                    log::info!("<INJECTION> Downloaded {}", injector);
                    message_sender.raw(&format!("Downloaded (from cdn) {}", injector));
                    log::info!("<INJECTION> Downloaded stable injector: {}", injector);
                }
                Err(e) => {
                    log::error!("<INJECTION> Failed to download {}: {}", injector, e);
                    message_sender.error(&format!("Failed to download {}: {}", injector, e));
                }
            }
        }
    }

    fn download_nightly_injectors(message_sender: Sender<String>) {
        let response =
            ureq::get("https://api.github.com/repos/AnarchyLoader/AnarchyInjector/releases")
                .call()
                .unwrap();

        let data: serde_json::Value = response.into_json().unwrap();

        let injector_names = vec!["AnarchyInjector_x86.exe", "AnarchyInjector_x64.exe"];
        for (index, injector_name) in injector_names.iter().enumerate() {
            let download_url = data
                .as_array()
                .unwrap()
                .iter()
                .find(|release| release["prerelease"].as_bool().unwrap_or(false))
                .and_then(|release| release["assets"].as_array())
                .and_then(|assets| assets.get(index))
                .and_then(|asset| asset["browser_download_url"].as_str())
                .unwrap_or("")
                .to_string();

            if download_url.is_empty() {
                log::error!(
                    "<INJECTION> Failed to get download URL for {}",
                    injector_name
                );
                message_sender.error(&format!("Failed to get download URL for {}", injector_name));
                continue;
            }

            if let Err(e) = download_file(&download_url, None) {
                log::error!("<INJECTION> Failed to download {}: {}", injector_name, e);
                message_sender.error(&format!("Failed to download {}: {}", injector_name, e));
            } else {
                message_sender.raw(&format!("Downloaded (nightly) {}", injector_name));
                log::info!("<INJECTION> Downloaded nightly injector: {}", injector_name);
            }
        }
    }

    pub fn manual_map_inject(
        dll_path: Option<PathBuf>,
        target_process: &str,
        message_sender: Sender<String>,
        status_message: Arc<Mutex<String>>,
        use_x64: bool,
        in_progress: Arc<AtomicBool>,
    ) -> bool {
        let dll_path_clone = match dll_path {
            Some(path) => path,
            None => {
                message_sender.error("DLL path is missing.");
                change_status_message(&status_message, "DLL path is missing.");
                log::error!("<INJECTION> DLL path is missing.");
                return false;
            }
        };

        let is_cs2 = target_process.eq_ignore_ascii_case("cs2.exe");
        let is_rust = target_process.eq_ignore_ascii_case("RustClient.exe");
        let injector_process = if is_cs2 || is_rust || use_x64 {
            "AnarchyInjector_x64.exe"
        } else {
            "AnarchyInjector_x86.exe"
        };

        log::debug!("<INJECTION> Using {} injector", injector_process);
        if use_x64 {
            log::debug!("<INJECTION> Forcing x64 injector");
        }

        let file_path = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("anarchyloader")
            .join(injector_process);

        if !file_path.exists() {
            match download_file(injector_process, None) {
                Ok(_) => {
                    log::debug!("<INJECTION> Downloaded manual map injector");
                }
                Err(e) => {
                    let error_message = format!("Failed to download manual map injector: {}", e);
                    message_sender.error(&error_message.clone());
                    log::error!("<INJECTION> {}", error_message);
                    change_status_message(&status_message, &error_message);

                    return false;
                }
            }
        }

        let mut command = Command::new(file_path);

        if dll_path_clone.file_name().unwrap() != "skeet.dll" {
            command.arg(target_process);
        } else {
            change_status_message(&status_message, "Please launch Counter-Strike.");
        }

        command
            .arg(dll_path_clone.clone())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        log::debug!("<INJECTION> Executing injector: {:?}", command);

        match command.spawn() {
            Ok(mut child) => {
                let stdout = child.stdout.take().unwrap();
                let stdout_reader = BufReader::new(stdout);
                let dll_name = dll_path_clone.clone();
                let stdout_thread = thread::spawn(move || {
                    for line in stdout_reader.lines() {
                        match line {
                            Ok(line) => {
                                if line.trim().is_empty() {
                                    continue;
                                }

                                let log_message = line.replace("\n", "\n<INJECTION> ");
                                log::info!("<INJECTION> {}", log_message);
                            }
                            Err(e) => log::error!("<INJECTION> Error reading stdout: {}", e),
                        }
                    }
                });

                let stderr = child.stderr.take().unwrap();
                let stderr_reader = BufReader::new(stderr);
                let message_sender_clone = message_sender.clone();
                let status_message_clone = status_message.clone();

                let stderr_thread = thread::spawn(move || {
                    let mut full_error = String::new();
                    for line in stderr_reader.lines() {
                        match line {
                            Ok(line) => {
                                full_error.push_str(&line);
                                log::error!("<INJECTION> {}", line);
                            }
                            Err(e) => log::error!("<INJECTION> Error reading stderr: {}", e),
                        }
                    }

                    if !full_error.is_empty() {
                        if full_error.contains("Can not find process") {
                            full_error += ", try running loader as admin.";
                        }
                        message_sender_clone.error(&full_error.clone());
                        change_status_message(
                            &status_message_clone,
                            &format!("Failed to execute injector: {}", full_error),
                        );
                    }
                });
                let in_progress_clone_wait = in_progress.clone();

                match child.wait() {
                    Ok(status) => {
                        stdout_thread.join().unwrap();
                        stderr_thread.join().unwrap();

                        if status.success() && in_progress_clone_wait.load(Ordering::SeqCst) {
                            let dll = dll_name.file_name().unwrap().to_string_lossy();
                            if !dll.starts_with("steam_") {
                                message_sender.success(&format!("{}", &dll));
                                log::info!("<INJECTION> Injected into {}", target_process);
                                change_status_message(&status_message, "Injection successful.");
                            }
                            true
                        } else {
                            false
                        }
                    }
                    Err(e) => {
                        let error_message = format!("Failed to wait for injector: {}", e);
                        message_sender.error(&error_message.clone());
                        log::error!("<INJECTION> {}", error_message);
                        change_status_message(&status_message, &error_message);

                        false
                    }
                }
            }
            Err(e) => {
                let error_message = format!("Failed to spawn injector: {}", e);
                message_sender.error(&error_message.clone());
                log::error!("<INJECTION> {}", error_message);
                change_status_message(&status_message, &error_message);

                false
            }
        }
    }

    pub fn inject_steam_module(
        &mut self,
        hack: Arc<Hack>,
        ctx: egui::Context,
        message_sender: Sender<String>,
    ) {
        if !hack.steam_module {
            message_sender.error("Selected hack does not have a steam module.");
            log::error!("<INJECTION> Selected hack does not have a steam module.");
            return;
        }

        let in_progress = Arc::clone(&self.communication.in_progress);
        let status_message = Arc::clone(&self.communication.status_message);
        let ctx_clone = ctx.clone();
        let message_sender_clone = message_sender.clone();
        let hack_clone = hack.clone();

        change_status_message(&status_message, "Starting steam module injection...");
        log::info!(
            "<INJECTION> Starting steam module injection for hack: {}",
            hack.name
        );

        in_progress.store(true, Ordering::SeqCst);

        ctx.send_viewport_cmd(ViewportCommand::EnableButtons {
            close: false,
            minimized: true,
            maximize: true,
        });

        thread::Builder::new()
            .name("SteamModuleInjectionThread".to_string())
            .spawn(move || {
                let steam_module_path = hack_clone
                    .file_path
                    .parent()
                    .unwrap()
                    .join(format!("steam_{}", hack_clone.file));

                if !steam_module_path.exists() {
                    if !Self::check_and_cancel(&in_progress, &status_message, &ctx_clone) {
                        return;
                    }
                    change_status_message(
                        &status_message,
                        &format!("Downloading steam module for {}...", hack_clone.name),
                    );

                    log::info!(
                        "<INJECTION> Steam module required for hack: {}",
                        hack_clone.name
                    );

                    match hack_clone.download_steam_module() {
                        Ok(_) => {
                            change_status_message(&status_message, "Downloaded steam module.");

                            log::debug!(
                                "<INJECTION> Downloaded steam module for {}",
                                hack_clone.name
                            );
                        }
                        Err(e) => {
                            in_progress.store(false, Ordering::SeqCst);
                            change_status_message(&status_message, &e.to_string());

                            log::error!("<INJECTION> Failed to download steam module: {}", e);
                            message_sender_clone
                                .error(&format!("Failed to download steam module: {}", e));
                            return;
                        }
                    }
                }

                if !Self::check_and_cancel(&in_progress, &status_message, &ctx_clone) {
                    return;
                }

                change_status_message(&status_message, "Injecting steam module...");

                log::info!(
                    "<INJECTION> Injecting steam module for hack: {}",
                    hack_clone.name
                );

                if MyApp::manual_map_inject(
                    Some(steam_module_path),
                    STEAM_EXE,
                    message_sender_clone.clone(),
                    status_message.clone(),
                    false,
                    in_progress.clone(),
                ) {
                    change_status_message(
                        &status_message,
                        "Steam module injected. Please launch Counter-Strike.",
                    );

                    message_sender_clone.raw("Steam module injected successfully!");
                    log::info!("<INJECTION> Steam module injected successfully!");
                } else {
                    in_progress.store(false, Ordering::SeqCst);
                    change_status_message(&status_message, "Failed to inject steam module.");

                    message_sender_clone.error("Failed to inject steam module.");
                    log::error!("<INJECTION> Failed to inject steam module.");
                    return;
                }

                in_progress.store(false, Ordering::SeqCst);
            })
            .expect("Failed to spawn steam module injection thread");
    }

    pub fn injection(
        &mut self,
        selected: Hack,
        ctx: egui::Context,
        message_sender: Sender<String>,
        force_x64: bool,
        inject_steam_module_only: bool,
    ) {
        if inject_steam_module_only {
            let hack_arc = Arc::new(selected);
            self.inject_steam_module(hack_arc, ctx, message_sender);
            return;
        }

        let in_progress = Arc::clone(&self.communication.in_progress);
        let status_message = Arc::clone(&self.communication.status_message);
        let selected_clone = selected.clone();
        let ctx_clone = ctx.clone();
        let skip_inject_delay = self.app.config.skip_injects_delay;
        let message_sender_clone = message_sender.clone();
        let is_cs2_or_csgo = selected.process.eq_ignore_ascii_case("cs2.exe")
            || selected.process.eq_ignore_ascii_case("csgo.exe");
        let automatically_run_game = self.app.config.automatically_run_game;
        let immediately_inject = self.app.config.immediately_inject_hack;

        change_status_message(&status_message, "Starting injection...");
        log::info!("<INJECTION> Starting injection for hack: {}", selected.name);

        in_progress.store(true, Ordering::SeqCst);
        let steam_module_injected = Arc::new(Mutex::new(false));

        ctx.send_viewport_cmd(ViewportCommand::EnableButtons {
            close: false,
            minimized: true,
            maximize: true,
        });

        thread::Builder::new()
            .name("InjectionThread".to_string())
            .spawn(move || {
                if automatically_run_game
                    && is_cs2_or_csgo
                    && !selected_clone.steam_module
                    && !is_process_running(&selected.process)
                    && !selected.steam_module
                {
                    if let Err(e) = start_cs_prompt() {
                        message_sender_clone.error(&format!(
                            "Failed to start Counter-Strike automatically: {}",
                            e
                        ));
                        log::error!(
                            "<INJECTION> Failed to start Counter-Strike automatically: {}",
                            e
                        );
                    }

                    let mut system = System::new_all();

                    loop {
                        if !Self::check_and_cancel(&in_progress, &status_message, &ctx_clone) {
                            return;
                        }

                        system.refresh_all();
                        if is_process_running(&selected_clone.process) {
                            thread::sleep(Duration::from_secs(10));
                            break;
                        }
                    }
                }

                if !selected_clone.file_path.exists() && !selected_clone.local {
                    change_status_message(
                        &status_message,
                        &format!("Downloading {}...", selected_clone.name),
                    );

                    log::info!(
                        "<INJECTION> Hack file not found, downloading: {}",
                        selected_clone.name
                    );

                    match selected_clone
                        .download(selected_clone.file_path.to_string_lossy().to_string())
                    {
                        Ok(_) => {
                            change_status_message(&status_message, "Downloaded.");

                            log::debug!("<INJECTION> Downloaded {}", selected_clone.name);
                        }
                        Err(e) => {
                            in_progress.store(false, Ordering::SeqCst);
                            change_status_message(&status_message, &e.to_string());

                            log::error!("<INJECTION> Failed to download hack file: {}", e);
                            message_sender_clone.error(&format!("Failed to download: {}", e));
                            return;
                        }
                    }
                }

                if !skip_inject_delay {
                    thread::sleep(Duration::from_secs(1));
                }

                let steam_module_injected_clone = steam_module_injected.clone();
                if selected_clone.steam_module {
                    let steam_module_path = selected_clone
                        .file_path
                        .parent()
                        .unwrap()
                        .join(format!("steam_{}", selected_clone.file));

                    if !steam_module_path.exists() {
                        if !Self::check_and_cancel(&in_progress, &status_message, &ctx_clone) {
                            return;
                        }
                        change_status_message(
                            &status_message,
                            &format!("Downloading steam module for {}...", selected_clone.name),
                        );

                        log::info!(
                            "<INJECTION> Steam module required for hack: {}",
                            selected_clone.name
                        );

                        match selected_clone.download_steam_module() {
                            Ok(_) => {
                                change_status_message(&status_message, "Downloaded steam module.");

                                log::debug!(
                                    "<INJECTION> Downloaded steam module for {}",
                                    selected_clone.name
                                );
                            }
                            Err(e) => {
                                in_progress.store(false, Ordering::SeqCst);
                                change_status_message(&status_message, &e.to_string());

                                log::error!("<INJECTION> Failed to download steam module: {}", e);
                                message_sender_clone
                                    .error(&format!("Failed to download steam module: {}", e));
                                return;
                            }
                        }
                    }

                    if !Self::check_and_cancel(&in_progress, &status_message, &ctx_clone) {
                        return;
                    }
                    change_status_message(&status_message, "Injecting steam module...");

                    log::info!(
                        "<INJECTION> Injecting steam module for hack: {}",
                        selected_clone.name
                    );

                    if MyApp::manual_map_inject(
                        Some(steam_module_path),
                        STEAM_EXE,
                        message_sender_clone.clone(),
                        status_message.clone(),
                        false,
                        in_progress.clone(),
                    ) {
                        *steam_module_injected_clone.lock().unwrap() = true;
                        change_status_message(
                            &status_message,
                            "Steam module injected. Please launch Counter-Strike.",
                        );

                        message_sender_clone.raw("Waiting for user to launch the game...");
                        log::info!("<INJECTION> Steam module injected, waiting for game launch.");

                        let mut system = System::new_all();

                        loop {
                            if !Self::check_and_cancel(&in_progress, &status_message, &ctx_clone) {
                                return;
                            }

                            system.refresh_all();
                            if is_process_running(&selected_clone.process) {
                                thread::sleep(Duration::from_secs(10));
                                break;
                            }
                        }
                    } else {
                        in_progress.store(false, Ordering::SeqCst);
                        change_status_message(&status_message, "Failed to inject steam module.");

                        return;
                    }
                }

                if !skip_inject_delay {
                    thread::sleep(Duration::from_secs(1));
                }

                let process_name = selected_clone.process.clone();
                let mut client_dll_found = false;
                let start_time = std::time::Instant::now();

                if !is_process_running(&selected_clone.process) {
                    in_progress.store(false, Ordering::SeqCst);
                    let error_message = format!(
                        "Failed to find process {}, try running loader as admin.",
                        &selected_clone.process
                    );
                    message_sender_clone.error(&error_message);
                    change_status_message(&status_message, &error_message);

                    log::error!("<INJECTION> {}", error_message);
                    return;
                }

                if !immediately_inject && is_cs2_or_csgo {
                    while !client_dll_found && start_time.elapsed() < Duration::from_secs(60) {
                        if !Self::check_and_cancel(&in_progress, &status_message, &ctx_clone) {
                            return;
                        }

                        if let Ok(process) = Process::with_name(&process_name) {
                            if let Ok(_module) = process.module(CLIENT_DLL) {
                                client_dll_found = true;
                                break;
                            } else {
                                log::warn!("<INJECTION> Failed to get process modules, retrying.");
                            }
                        } else {
                            log::warn!("<INJECTION> Process not found, retrying.");
                        }

                        if !client_dll_found {
                            thread::sleep(Duration::from_secs(1));
                        }
                    }

                    if !client_dll_found {
                        in_progress.store(false, Ordering::SeqCst);
                        let error_message =
                            "client.dll not found after 60 seconds, injection aborted.";
                        message_sender_clone.error(error_message);
                        change_status_message(&status_message, error_message);

                        log::error!("<INJECTION> {}", error_message);
                        return;
                    }

                    log::info!("<INJECTION> client.dll found, waiting 10 seconds...");
                    change_status_message(
                        &status_message,
                        "Found client.dll, waiting 10 seconds...",
                    );

                    thread::sleep(Duration::from_secs(10));
                }

                if !Self::check_and_cancel(&in_progress, &status_message, &ctx_clone) {
                    return;
                }

                change_status_message(&status_message, "Injecting...");

                log::info!("<INJECTION> Injecting hack: {}", selected_clone.name);

                if !skip_inject_delay {
                    thread::sleep(Duration::from_secs(1));
                }

                let dll_path = Some(selected_clone.file_path.clone());
                let target_process = &selected_clone.process;
                let status_message_clone = status_message.clone();

                log::debug!("<INJECTION> Hack details: {:?}", selected_clone);

                if MyApp::manual_map_inject(
                    dll_path,
                    target_process,
                    message_sender_clone.clone(),
                    status_message_clone,
                    if selected_clone.arch == "x64" {
                        true
                    } else {
                        force_x64
                    },
                    in_progress.clone(),
                ) {
                    *steam_module_injected.lock().unwrap() = false;
                }

                in_progress.store(false, Ordering::SeqCst);
            })
            .expect("Failed to spawn injection thread");
    }

    fn check_and_cancel(
        in_progress: &Arc<AtomicBool>,
        status_message: &Arc<Mutex<String>>,
        ctx: &egui::Context,
    ) -> bool {
        if !in_progress.load(Ordering::SeqCst) {
            change_status_message(status_message, "Injection cancelled.");
            ctx.send_viewport_cmd(ViewportCommand::EnableButtons {
                close: true,
                minimized: true,
                maximize: true,
            });

            false
        } else {
            true
        }
    }
}
