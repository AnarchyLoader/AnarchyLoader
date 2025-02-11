// downloader.rs
use std::{fs::File, io::copy, path::Path};

use super::config::Config;

/// Downloads a file from the CDN or URL, saving it to the loader directory, or the specified directory.
pub fn download_file(
    file: &str,
    dest_dir: Option<&Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    if file.starts_with("http://") || file.starts_with("https://") {
        log::info!("<DOWNLOAD> Downloading {} from URL...", file);

        match ureq::get(file).call() {
            Ok(resp) if resp.status() == 200 => {
                log::info!("<DOWNLOAD> Downloading {} from URL...", file);
                let file_name = Path::new(file)
                    .file_name()
                    .ok_or_else(|| format!("Invalid URL: {}", file))?
                    .to_string_lossy();

                let dest_path = if let Some(dir) = dest_dir {
                    dir.join(file_name.as_ref())
                } else {
                    dirs::config_dir()
                        .unwrap_or_else(|| std::path::PathBuf::from("."))
                        .join("anarchyloader")
                        .join(file_name.as_ref())
                };

                log::info!("Downloading to: {:?}", dest_path);
                log::info!("Destination directory exists: {}", dest_path.exists());

                let mut dest_file = File::create(dest_path)?;
                copy(&mut resp.into_reader(), &mut dest_file)?;
                return Ok(());
            }
            Ok(resp) if resp.status() == 404 => {
                return Err(format!("File not found at URL: {}", file).into());
            }
            Ok(resp) => {
                log::warn!(
                    "<DOWNLOAD> Failed to download {} from URL: {}",
                    file,
                    resp.status()
                );
            }
            Err(e) => {
                log::warn!("<DOWNLOAD> Failed to download {} from URL: {}", file, e);
            }
        }
        Err(format!("Failed to download {} from URL.", file).into())
    } else {
        let config = Config::load();

        let mut endpoints = vec![config.cdn_endpoint];
        endpoints.extend(config.cdn_extra_endpoints);

        for (i, endpoint) in endpoints.iter().enumerate() {
            let url = format!("{}{}", endpoint, file);
            log::info!("<DOWNLOAD> Downloading {} from CDN {}...", file, i + 1);
            match ureq::get(&url).call() {
                Ok(resp) if resp.status() == 200 => {
                    let dest_path = dirs::config_dir()
                        .unwrap_or_else(|| std::path::PathBuf::from("."))
                        .join("anarchyloader")
                        .join(file);

                    log::info!("Downloading to: {:?}", dest_path);
                    let mut dest_file = File::create(dest_path)?;

                    log::info!(
                        "<DOWNLOAD> Downloaded {} successfully from CDN {}.",
                        file,
                        i + 1
                    );
                    copy(&mut resp.into_reader(), &mut dest_file)?;
                    return Ok(());
                }
                Ok(resp) if resp.status() == 404 => {
                    return Err(format!("File not found: {}", file).into());
                }
                Ok(resp) => {
                    log::warn!(
                        "<DOWNLOAD> Failed to download {} from CDN {}: {}",
                        file,
                        i + 1,
                        resp.status()
                    );
                }
                Err(e) => {
                    log::warn!(
                        "<DOWNLOAD> Failed to download {} from CDN {}: {}",
                        file,
                        i + 1,
                        e
                    );
                }
            }
        }

        Err(format!("Failed to download {} from all CDN endpoints.", file).into())
    }
}

#[allow(dead_code)]
// TODO: use it in the future
pub(crate) fn extract_zip(archive_path: &Path, destination: &Path) -> Result<(), String> {
    zip_extract::extract(
        File::open(archive_path).map_err(|e| e.to_string())?,
        destination,
        true,
    )
        .map_err(|e| e.to_string())
}
