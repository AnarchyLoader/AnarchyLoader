use std::{
    process::Command,
    sync::{mpsc::Sender, Arc, Mutex},
    thread,
};

use crate::{hacks::Hack, utils::downloader, MyApp};

impl MyApp {
    /// Download the roblox zip
    pub fn download_executor() -> Result<(), Box<dyn std::error::Error>> {
        downloader::download_file("roblox.zip")?;
        Ok(())
    }

    /// Extract the downloaded roblox zip
    pub fn extract_executor(
        status_message: Arc<Mutex<String>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut status = status_message.lock().unwrap();
        *status = "Extracting...".to_string();

        let app_path = dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("anarchyloader");

        let file_path = app_path.join("roblox.zip");

        let dest_path = app_path.join("roblox");

        if !std::path::Path::new(&dest_path).exists() {
            if let Err(e) = zip_extract::extract(std::fs::File::open(&file_path)?, &dest_path, true)
            {
                log::error!("Failed to extract file: {}", e);
                std::fs::remove_file(&file_path)?;
                return Err(Box::new(e));
            }
        }

        if let Err(e) = std::fs::remove_file(&file_path) {
            log::error!("Failed to delete zip file: {}", e);
            return Err(Box::new(e));
        }

        Ok(())
    }

    pub fn run_executor(&self, selected: Hack, ctx: egui::Context, message_sender: Sender<String>) {
        let in_progress = Arc::clone(&self.communication.in_progress);
        let selected_clone = selected.clone();
        let status_message = Arc::clone(&self.communication.status_message);
        let ctx_clone = ctx.clone();
        let message_sender_clone = message_sender.clone();
        let app_path = dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("anarchyloader");
        let folder_path = app_path.join("roblox");
        let zip_path = app_path.join("roblox.zip");

        {
            let mut status = status_message.lock().unwrap();
            *status = "Running...".to_string();
        }

        in_progress.store(true, std::sync::atomic::Ordering::SeqCst);

        thread::spawn(move || {
            ctx_clone.request_repaint();

            if !folder_path.exists() {
                {
                    let mut status = status_message.lock().unwrap();
                    *status = "Downloading...".to_string();
                }
                ctx_clone.request_repaint();

                match MyApp::download_executor() {
                    Ok(_) => {
                        let mut status = status_message.lock().unwrap();
                        *status = "Downloaded.".to_string();
                        log::debug!("Downloaded executor");
                        ctx_clone.request_repaint();
                    }
                    Err(e) => {
                        let mut status = status_message.lock().unwrap();
                        *status = format!("{}", e);
                        in_progress.store(false, std::sync::atomic::Ordering::SeqCst);
                        log::error!("Failed to download: {}", e);
                        ctx_clone.request_repaint();
                        let _ = message_sender_clone.send(format!("Failed to download: {}", e));
                    }
                }
            }

            if zip_path.exists() {
                if let Err(e) = MyApp::extract_executor(status_message.clone()) {
                    let mut status = status_message.lock().unwrap();
                    *status = format!("Failed to extract: {}", e);
                    in_progress.store(false, std::sync::atomic::Ordering::SeqCst);
                    log::error!("Failed to extract: {}", e);
                    ctx_clone.request_repaint();
                    let _ = message_sender_clone.send(format!("Failed to extract: {}", e));
                }
            }

            let mut status = status_message.lock().unwrap();
            *status = "Running...".to_string();

            let executor_path = app_path.join("roblox").join(&selected_clone.file);

            let status = Command::new("cmd")
                .current_dir(app_path.join("roblox"))
                .arg("/C")
                .arg("start")
                .arg(format!("{}", executor_path.display()))
                .status();

            match status {
                Ok(_) => {
                    log::info!("New console launched");
                    std::process::exit(0);
                }
                Err(e) => {
                    let mut status = status_message.lock().unwrap();
                    *status = format!("Failed to run: {}", e);
                    in_progress.store(false, std::sync::atomic::Ordering::SeqCst);
                    log::error!("Failed to run: {}", e);
                    ctx_clone.request_repaint();
                    let _ = message_sender_clone.send(format!("Failed to run: {}", e));
                }
            }
        });
    }
}
