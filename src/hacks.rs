use serde::Deserialize;

use crate::downloader::download_file;

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub(crate) struct HackApiResponse {
    pub name: String,
    pub description: String,
    pub author: String,
    pub status: String,
    pub file: String,
    pub process: String,
    pub source: String,
    pub game: String,
}

#[derive(Clone, PartialEq)]
pub(crate) struct Hack {
    pub name: String,
    pub description: String,
    pub author: String,
    pub status: String,
    pub file: String,
    pub process: String,
    pub source: String,
    pub game: String,
    pub file_path: std::path::PathBuf,
}

impl Hack {
    pub(crate) fn new(
        name: &str,
        description: &str,
        author: &str,
        status: &str,
        file: &str,
        process: &str,
        source: &str,
        game: &str,
    ) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            author: author.to_string(),
            status: status.to_string(),
            file: file.to_string(),
            process: process.to_string(),
            source: source.to_string(),
            game: game.to_string(),
            file_path: dirs::config_dir()
                .unwrap_or_else(|| std::path::PathBuf::from("."))
                .join("anarchyloader")
                .join(&file),
        }
    }

    pub(crate) fn download(&self, file_path: String) -> Result<(), String> {
        if !std::path::Path::new(&file_path).exists() {
            println!("Downloading {}...", self.name);
            match download_file(&self.file, &file_path) {
                Ok(_) => {
                    println!("Downloaded {}!", self.name);
                    Ok(())
                }
                Err(e) => Err(format!("Failed to download file: {}", e)),
            }
        } else {
            Ok(())
        }
    }
}
