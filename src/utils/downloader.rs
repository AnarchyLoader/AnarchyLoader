use std::{fs::File, io::copy};

use super::config::Config;

/// Downloads a file from the CDN, saving it to the loader directory.
pub fn download_file(file: &str) -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load();

    let endpoints = &[config.cdn_endpoint, config.cdn_fallback_endpoint];

    for (i, endpoint) in endpoints.iter().enumerate() {
        let url = format!("{}{}", endpoint, file);
        log::info!("Downloading {} from CDN {}...", file, i + 1);
        match ureq::get(&url).call() {
            Ok(resp) if resp.status() == 200 => {
                log::info!("Downloaded {} successfully from CDN {}.", file, i + 1);
                let mut dest_file = File::create(
                    dirs::config_dir()
                        .unwrap_or_else(|| std::path::PathBuf::from("."))
                        .join("anarchyloader")
                        .join(file),
                )?;
                copy(&mut resp.into_reader(), &mut dest_file)?;
                return Ok(());
            }
            Ok(resp) if resp.status() == 404 => {
                return Err(format!("File not found: {}", file).into());
            }
            Ok(resp) => {
                log::warn!(
                    "Failed to download {} from CDN {}: {}",
                    file,
                    i + 1,
                    resp.status()
                );
            }
            Err(e) => {
                log::warn!("Failed to download {} from CDN {}: {}", file, i + 1, e);
            }
        }
    }

    Err(format!("Failed to download {} from all CDN endpoints.", file).into())
}
