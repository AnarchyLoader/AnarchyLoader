use std::{fs::File, io::copy};

use reqwest;

use crate::config::Config;

pub fn download_file(file: &str, destination: &str) -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load_config();
    let response = reqwest::blocking::get(format!("{}{}", config.cdn_endpoint, file))?;

    if response.status().is_success() {
        let mut file = File::create(destination)?;

        let content = response.bytes()?;
        copy(&mut content.as_ref(), &mut file)?;

        Ok(())
    } else {
        Err(format!("Failed to download file: {}", response.status()).into())
    }
}
