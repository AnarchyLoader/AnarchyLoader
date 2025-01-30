use std::{fs::File, io::{copy, Read, Write}, thread, time::Duration};

use super::config::Config;

/// Downloads a file from the CDN or URL, saving it to the loader directory.
pub fn download_file(file: &str) -> Result<(), Box<dyn std::error::Error>> {
    const MAX_RETRIES: u32 = 5;
    const BACKOFF_BASE: u64 = 2;

    if file.starts_with("https://") {
        log::info!("Downloading {} from URL...", file);

        for attempt in 0..=MAX_RETRIES {
            match ureq::get(file).call() {
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

            if attempt < MAX_RETRIES {
                let backoff = BACKOFF_BASE.pow(attempt) * 100;
                log::info!("Retrying in {} ms...", backoff);
                thread::sleep(Duration::from_millis(backoff));
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

            for attempt in 0..=MAX_RETRIES {
                match ureq::get(&url).call() {
                    Ok(resp) if resp.status() == 200 => {
                        log::info!("Downloaded {} successfully from CDN {}.", file, i + 1);
                        let mut dest_file = File::create(
                            dirs::config_dir()
                                .unwrap_or_else(|| std::path::PathBuf::from("."))
                                .join("anarchyloader")
                                .join(file),
                        )?;
                        let mut reader = resp.into_reader();
                        let total_size = reader
                            .headers()
                            .get("Content-Length")
                            .and_then(|s| s.parse::<u64>().ok())
                            .unwrap_or(0);
                        let mut buffer = [0; 8192];
                        let mut downloaded = 0;

                        while let Ok(bytes_read) = reader.read(&mut buffer) {
                            if bytes_read == 0 {
                                break;
                            }
                            dest_file.write_all(&buffer[..bytes_read])?;
                            downloaded += bytes_read as u64;
                            log::info!(
                                "Downloading {}: {} / {} bytes",
                                file,
                                downloaded,
                                total_size
                            );
                        }

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

                if attempt < MAX_RETRIES {
                    let backoff = BACKOFF_BASE.pow(attempt) * 100;
                    log::info!("Retrying in {} ms...", backoff);
                    thread::sleep(Duration::from_millis(backoff));
                }
            }
        }

        Err(format!("Failed to download {} from all CDN endpoints.", file).into())
    }
}
