use std::{fs::File, io::copy};

use crate::config::Config;

pub fn download_file(file: &str, destination: &str) -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load_config();
    let url = format!("{}{}", config.cdn_endpoint, file);
    let response = ureq::get(&url).call()?;

    if response.status() == 200 {
        let mut file = File::create(destination)?;
        let mut reader = response.into_reader();
        copy(&mut reader, &mut file)?;
        Ok(())
    } else {
        Err(format!("Cannot download file: {}", response.status()).into())
    }
}
