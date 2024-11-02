use std::{fs::File, io::copy};

use reqwest;

pub fn download_file(file: &str, destination: &str) -> Result<(), Box<dyn std::error::Error>> {
    let response = reqwest::blocking::get(format!(
        "{}/{}",
        "https://cdn.collapseloader.org/anarchy/", file
    ))?;

    if response.status().is_success() {
        let mut file = File::create(destination)?;

        let content = response.bytes()?;
        copy(&mut content.as_ref(), &mut file)?;

        Ok(())
    } else {
        Err(format!("Failed to download file: {}", response.status()).into())
    }
}
