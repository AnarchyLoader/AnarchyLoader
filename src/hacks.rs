use crate::downloader::download_file;

#[derive(Clone, PartialEq)]
pub(crate) struct Hack {
    pub name: String,
    pub description: String,
    pub author: String,
    pub status: String,
    pub file: String,
    pub process: String,
}

impl Hack {
    pub(crate) fn new(
        name: &str,
        description: &str,
        author: &str,
        status: &str,
        file: &str,
        process: &str,
    ) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            author: author.to_string(),
            status: status.to_string(),
            file: file.to_string(),
            process: if process.is_empty() {
                "hl.exe".to_string()
            } else {
                process.to_string()
            },
        }
    }

    pub(crate) fn download(&self, status_message: &mut String, file_path: String) {
        println!("Downloading {}...", self.name);

        if !std::path::Path::new(&file_path).exists() {
            match download_file(&self.name, &file_path) {
                Ok(_) => {
                    status_message.clear();
                }
                Err(e) => {
                    *status_message = format!("Failed to download file: {}", e);
                    return;
                }
            }
        }
    }
}
