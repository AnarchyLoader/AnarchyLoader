use std::{
    process::Command,
    sync::{mpsc::Sender, Arc, Mutex},
    thread,
};

use crate::{
    inject::change_status_message,
    utils::api::{downloader, hacks::Hack},
    MyApp,
};

pub struct Roblox {}

impl Roblox {
    /// Download the roblox zip
    pub fn download_executor() -> Result<(), Box<dyn std::error::Error>> {
        log::info!("<ROBLOX> Downloading roblox executor...");
        downloader::download_file("roblox.zip", None)?;
        log::info!("<ROBLOX> Roblox executor downloaded successfully.");
        Ok(())
    }

    /// Extract the downloaded roblox zip
    pub fn extract_executor(
        status_message: Arc<Mutex<String>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        change_status_message(&status_message, "Extracting...");
        log::info!("<ROBLOX> Extracting roblox executor...");

        let app_path = dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("anarchyloader");

        let file_path = app_path.join("roblox.zip");

        let dest_path = app_path.join("roblox");

        if !std::path::Path::new(&dest_path).exists() {
            if let Err(e) = zip_extract::extract(std::fs::File::open(&file_path)?, &dest_path, true)
            {
                log::error!("<ROBLOX> Failed to extract file: {}", e);
                std::fs::remove_file(&file_path)?;
                return Err(Box::new(e));
            }
        }

        if let Err(e) = std::fs::remove_file(&file_path) {
            log::error!("<ROBLOX> Failed to delete zip file: {}", e);
            return Err(Box::new(e));
        }

        log::info!("<ROBLOX> Roblox executor extracted successfully.");
        Ok(())
    }
}

impl MyApp {
    pub fn run_executor(&self, selected: Hack, ctx: egui::Context, message_sender: Sender<String>) {
        let in_progress = Arc::clone(&self.communication.in_progress);
        let selected_clone = selected.clone();
        let status_message = Arc::clone(&self.communication.status_message);
        let ctx_clone = ctx.clone();
        let message_sender_clone = message_sender.clone();
        let folder_path = self.app.meta.path.join("roblox");
        let zip_path = self.app.meta.path.join("roblox.zip");
        let app_path_clone = self.app.meta.path.clone();

        change_status_message(&status_message, "Running...");
        log::info!(
            "<ROBLOX> Running roblox executor for hack: {}",
            selected.name
        );

        in_progress.store(true, std::sync::atomic::Ordering::SeqCst);

        thread::spawn(move || {
            ctx_clone.request_repaint();

            if !folder_path.exists() {
                change_status_message(&status_message, "Downloading...");
                ctx_clone.request_repaint();
                log::info!("<ROBLOX> Roblox executor folder not found, downloading...");

                match Roblox::download_executor() {
                    Ok(_) => {
                        change_status_message(&status_message, "Downloaded.");
                        log::debug!("<ROBLOX> Downloaded executor");
                        ctx_clone.request_repaint();
                    }
                    Err(e) => {
                        change_status_message(&status_message, &format!("{}", e));
                        in_progress.store(false, std::sync::atomic::Ordering::SeqCst);
                        log::error!("<ROBLOX> Failed to download executor: {}", e);
                        ctx_clone.request_repaint();
                        let _ = message_sender_clone.send(format!("Failed to download: {}", e));
                        return;
                    }
                }
            }

            if zip_path.exists() {
                if let Err(e) = Roblox::extract_executor(status_message.clone()) {
                    change_status_message(&status_message, &format!("Failed to extract: {}", e));
                    in_progress.store(false, std::sync::atomic::Ordering::SeqCst);
                    log::error!("<ROBLOX> Failed to extract executor: {}", e);
                    ctx_clone.request_repaint();
                    let _ = message_sender_clone.send(format!("Failed to extract: {}", e));
                    return;
                }
            }

            change_status_message(&status_message, "Running...");

            let executor_path = app_path_clone.join("roblox").join(&selected_clone.file);

            let status = Command::new("cmd")
                .current_dir(app_path_clone.join("roblox"))
                .arg("/C")
                .arg("start")
                .arg(format!("{}", executor_path.display()))
                .status();

            match status {
                Ok(_) => {
                    log::info!("<ROBLOX> New console launched for executor.");
                    std::process::exit(0);
                }
                Err(e) => {
                    change_status_message(&status_message, &format!("Failed to run: {}", e));
                    in_progress.store(false, std::sync::atomic::Ordering::SeqCst);
                    log::error!("<ROBLOX> Failed to run executor: {}", e);
                    ctx_clone.request_repaint();
                    let _ = message_sender_clone.send(format!("Failed to run: {}", e));
                }
            }
        });
    }
}
