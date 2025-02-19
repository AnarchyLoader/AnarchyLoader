use std::{
    path::PathBuf,
    process::Command,
    sync::{mpsc::Sender, Arc, Mutex},
    thread,
    time::Duration,
};

use eframe::egui::{self};

use crate::{
    utils::{api::downloader::download_file, ui::messages::MessageSender},
    Hack, MyApp,
};

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

        for injector in injectors {
            let injector_path = dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("anarchyloader")
                .join(injector);

            if injector_path.exists() {
                if let Err(e) = std::fs::remove_file(&injector_path) {
                    log::error!("<INJECTION> Failed to delete {} injector: {}", injector, e);
                    return Err(format!("Failed to delete {} injector: {}", injector, e));
                }
                log::info!("<INJECTION> Deleted {}", injector);
            }
        }
        Ok(())
    }

    pub fn download_injectors(&mut self, message_sender: Sender<String>, nightly: bool) {
        if nightly {
            let injectors = vec![0, 1];

            thread::spawn(move || {
                let response = ureq::get(
                    "https://api.github.com/repos/AnarchyLoader/AnarchyInjector/releases",
                )
                .call()
                .unwrap();

                let data: serde_json::Value = response.into_json().unwrap();

                for injector in injectors {
                    let injector_name = if injector == 0 {
                        "AnarchyInjector_x86.exe"
                    } else {
                        "AnarchyInjector_x64.exe"
                    };

                    let download_url = data
                        .as_array()
                        .unwrap()
                        .iter()
                        .find(|release| release["prerelease"].as_bool().unwrap_or(false))
                        .and_then(|release| release["assets"].as_array())
                        .and_then(|assets| assets.get(injector))
                        .and_then(|asset| asset["browser_download_url"].as_str())
                        .unwrap_or("")
                        .to_string();

                    if download_url.is_empty() {
                        log::error!(
                            "<INJECTION> Failed to get download URL for {}",
                            injector_name
                        );
                        message_sender
                            .error(&format!("Failed to get download URL for {}", injector_name));
                    }

                    if let Err(e) = download_file(&download_url, None) {
                        log::error!("<INJECTION> Failed to download {}: {}", injector_name, e);
                        message_sender
                            .error(&format!("Failed to download {}: {}", injector_name, e));
                    }

                    message_sender.raw(&format!("Downloaded (nightly) {}", injector_name));
                    log::info!("<INJECTION> Downloaded nightly injector: {}", injector_name);
                }
            });
        } else {
            let injectors = vec!["AnarchyInjector_x86.exe", "AnarchyInjector_x64.exe"];
            thread::spawn(move || {
                for injector in injectors {
                    match download_file(injector, None) {
                        Ok(_) => {
                            log::info!("<INJECTION> Downloaded {}", injector);
                            message_sender.raw(&format!("Downloaded (from cdn) {}", injector));
                            log::info!("<INJECTION> Downloaded stable injector: {}", injector);
                        }
                        Err(e) => {
                            log::error!("<INJECTION> Failed to download {}: {}", injector, e);
                            message_sender
                                .error(&format!("Failed to download {}: {}", injector, e));
                        }
                    }
                }
            });
        }
    }

    pub fn manual_map_inject(
        dll_path: Option<PathBuf>,
        target_process: &str,
        message_sender: Sender<String>,
        status_message: Arc<Mutex<String>>,
        ctx: egui::Context,
        use_x64: bool,
    ) {
        let dll_path_clone = dll_path.clone().unwrap();
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
                    ctx.request_repaint();
                    return;
                }
            }
        }

        let mut command = Command::new(file_path);
        command.arg(target_process).arg(dll_path.unwrap());

        log::debug!("<INJECTION> Executing injector: {:?}", command);

        let output = command.output();

        match output {
            Ok(output) => {
                if output.status.success() {
                    let stdout_message = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    let log_message = stdout_message.replace("\n", "\n<INJECTION> ");
                    log::info!("<INJECTION> {}", log_message);
                    message_sender.success(&dll_path_clone.file_name().unwrap().to_string_lossy());
                    log::info!("<INJECTION> Injected into {}", target_process);
                    change_status_message(&status_message, "Injection successful.");
                    ctx.request_repaint();
                } else {
                    let mut error_message =
                        String::from_utf8_lossy(&output.stderr).trim().to_string();
                    if error_message.contains("Can not find process") {
                        error_message += ", try running loader as admin.";
                    }
                    message_sender.error(&error_message.clone());
                    log::error!("<INJECTION> Failed to execute injector: {}", error_message);
                    change_status_message(
                        &status_message,
                        &format!("Failed to execute injector: {}", error_message),
                    );
                    ctx.request_repaint();
                }
            }
            Err(e) => {
                let error_message = format!("Failed to execute injector: {}", e);
                message_sender.error(&error_message.clone());
                log::error!("<INJECTION> {}", error_message);
                change_status_message(&status_message, &error_message);
                ctx.request_repaint();
            }
        }
    }

    // MARK: Manual map injection
    pub fn injection(
        &mut self,
        selected: Hack,
        ctx: egui::Context,
        message_sender: Sender<String>,
        force_x64: bool,
    ) {
        let in_progress = Arc::clone(&self.communication.in_progress);
        let status_message = Arc::clone(&self.communication.status_message);
        let selected_clone = selected.clone();
        let ctx_clone = ctx.clone();
        let skip_inject_delay = self.app.config.skip_injects_delay;
        let message_sender_clone = message_sender.clone();

        change_status_message(&status_message, "Starting injection...");
        log::info!("<INJECTION> Starting injection for hack: {}", selected.name);

        in_progress.store(true, std::sync::atomic::Ordering::SeqCst);

        thread::spawn(move || {
            ctx_clone.request_repaint();
            if !skip_inject_delay {
                thread::sleep(Duration::from_secs(1));
            }

            if !selected_clone.file_path.exists() && !selected_clone.local {
                change_status_message(
                    &status_message,
                    &format!("Downloading {}...", selected_clone.name),
                );
                ctx_clone.request_repaint();
                log::info!(
                    "<INJECTION> Hack file not found, downloading: {}",
                    selected_clone.name
                );

                match selected_clone
                    .download(selected_clone.file_path.to_string_lossy().to_string())
                {
                    Ok(_) => {
                        change_status_message(&status_message, "Downloaded.");
                        ctx_clone.request_repaint();
                        log::debug!("<INJECTION> Downloaded {}", selected_clone.name);
                    }
                    Err(e) => {
                        in_progress.store(false, std::sync::atomic::Ordering::SeqCst);
                        change_status_message(&status_message, &e.to_string());
                        ctx_clone.request_repaint();
                        log::error!("<INJECTION> Failed to download hack file: {}", e);
                        message_sender_clone.error(&format!("Failed to download: {}", e));
                        return;
                    }
                }
            }

            if !skip_inject_delay {
                thread::sleep(Duration::from_secs(1));
            }

            change_status_message(&status_message, "Injecting...");
            ctx_clone.request_repaint();
            log::info!("<INJECTION> Injecting hack: {}", selected_clone.name);

            if !skip_inject_delay {
                thread::sleep(Duration::from_secs(1));
            }

            let dll_path = Some(selected_clone.file_path.clone());
            let target_process = &selected_clone.process;
            let status_message_clone = status_message.clone();

            log::debug!("<INJECTION> Hack details: {:?}", selected_clone);

            MyApp::manual_map_inject(
                dll_path,
                target_process,
                message_sender_clone.clone(),
                status_message_clone,
                ctx_clone.clone(),
                if selected_clone.arch == "x64" {
                    true
                } else {
                    force_x64
                },
            );

            in_progress.store(false, std::sync::atomic::Ordering::SeqCst);
            ctx_clone.request_repaint();
            log::info!(
                "<INJECTION> Injection process completed for hack: {}",
                selected_clone.name
            );
        });
    }
}
