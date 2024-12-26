use std::{fs::File, io::copy};

use crate::config::Config;

pub fn download_file(file: &str, destination: &str) -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load();

    let mut url = format!("{}{}", config.cdn_endpoint, file);
    let mut response = ureq::get(&url).call();

    log::info!("Downloading {}...", file);

    if response.is_err() || response.as_ref().unwrap().status() != 200 {
        log::warn!("Primary CDN endpoint unavailable, trying fallback...");
        url = format!("{}{}", config.cdn_fallback_endpoint, file);
        response = ureq::get(&url).call();

        if response.is_err() {
            return Err(format!(
                "Failed to download from both CDN endpoints: {:?}",
                response.err()
            )
            .into());
        }
    }

    let response = response.unwrap();

    if response.status() == 200 {
        let mut file = File::create(destination)?;
        let mut reader = response.into_reader();
        copy(&mut reader, &mut file)?;
        Ok(())
    } else {
        Err(format!("Cannot download file: {}", response.status()).into())
    }
}
