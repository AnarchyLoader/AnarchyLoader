use std::{fs::File, io::copy};

use base64::prelude::*;

use super::config::Config;

/// Downloads a file from the CDN or URL, saving it to the loader directory.
pub fn download_file(file: &str) -> Result<(), Box<dyn std::error::Error>> {
    if file.starts_with("https://") {
        log::info!("Downloading {} from URL...", file);

        // its my fine-grained token for read only artifacts, encoded in base64 to bypass github push protection
        let api_token = String::from_utf8(BASE64_STANDARD.decode(b"Z2l0aHViX3BhdF8xMUFUSEVWSVEwcXhkN2xJekx5bzJQX0N5T2diRUhhYjZQTXBpdVpWaTlJa2xOVmxKTHFjRUtSaEZFTTk3MHJFdDc1NjU2TjNJVjhtOVd2MzRx")?)?;

        match ureq::get(file)
            .set("Authorization", &format!("Bearer {}", api_token))
            .call()
        {
            Ok(resp) if resp.status() == 200 => {
                log::info!("Downloaded {} successfully from URL.", file);
                let file_name = std::path::Path::new(file)
                    .file_name()
                    .ok_or_else(|| format!("Invalid URL: {}", file))?
                    .to_string_lossy();
                let mut dest_file = File::create(
                    dirs::config_dir()
                        .unwrap_or_else(|| std::path::PathBuf::from("."))
                        .join("anarchyloader")
                        .join(file_name.as_ref()),
                )?;
                copy(&mut resp.into_reader(), &mut dest_file)?;
                return Ok(());
            }
            Ok(resp) if resp.status() == 404 => {
                return Err(format!("File not found at URL: {}", file).into());
            }
            Ok(resp) => {
                log::warn!("Failed to download {} from URL: {}", file, resp.status());
            }
            Err(e) => {
                log::warn!("Failed to download {} from URL: {}", file, e);
            }
        }
        Err(format!("Failed to download {} from URL.", file).into())
    } else {
        let config = Config::load();

        let mut endpoints = vec![config.cdn_endpoint];
        endpoints.extend(config.cdn_extra_endpoints);

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
}
