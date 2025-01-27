use std::path::PathBuf;

use egui_modal::Modal;
use pelite::{pe, strings::Config};
#[cfg(feature = "scanner")]
use pelite::{
    pe64::{Pe, PeFile},
    FileMap,
};
use regex::Regex;

use crate::{utils::custom_widgets::Button, MyApp};

pub struct Scanner {
    pub file: PathBuf,
}

#[derive(Clone, Debug)]
pub struct ScannerPopup {
    pub dll: String,
    pub show_results: bool,
}

impl Default for ScannerPopup {
    fn default() -> Self {
        Self {
            dll: String::new(),
            show_results: false,
        }
    }
}

#[cfg(feature = "scanner")]
impl Scanner {
    pub fn new(file: PathBuf) -> Scanner {
        Scanner { file }
    }

    pub fn scan(&self, app_path: PathBuf) -> Result<(), String> {
        let mut output = String::new();

        let file_map = FileMap::open(self.file.as_path()).unwrap();

        let file = match pe::PeFile::from_bytes(file_map.as_ref()) {
            Ok(file) => file,
            Err(err) => return Err(format!("Failed to parse PE file: {}", err)),
        };

        output += "PE-File Information:\n";
        output += &format!("{:?}", file.section_headers());

        output += "\n\n====================\n";

        match self.scan_imports(&file) {
            Ok(import_output) => output += &import_output,
            Err(err) => output += &format!("Failed to scan imports: {}\n", err),
        }

        output += "====================\n\n";

        match self.scan_links(&file) {
            Ok(links_output) => output += &links_output,
            Err(err) => output += &format!("Failed to scan links: {}\n", err),
        }

        output += "====================\n\n";

        std::fs::write(app_path.join("scanner_results.txt"), output).unwrap();

        Ok(())
    }

    fn scan_imports(&self, file: &PeFile) -> Result<String, String> {
        let mut output = String::new();

        let imports = match file.imports() {
            Ok(imports) => imports,
            Err(err) => return Err(format!("Failed to read imports: {}", err)),
        };

        output += "Imports:\n";

        for desc in imports {
            let dll_name = desc.dll_name();
            if let Ok(iat) = desc.iat() {
                output += &format!(
                    "Imported {} functions from {}\n",
                    iat.len(),
                    dll_name.unwrap()
                );
            } else {
                output += &format!("Failed to read imports for {:?}\n", dll_name);
            }
        }

        Ok(output)
    }

    fn scan_links(&self, file: &PeFile) -> Result<String, String> {
        let mut output = String::new();

        output += "Links:\n";
        let config = Config::default();
        let url_regex =
            Regex::new(r"\b(?:https?|ftp|ssh|telnet|file)://[^\s/$.?#].[^\s]*\b").unwrap();
        let ip_regex = Regex::new(r"\b(?:\d{1,3}\.){3}\d{1,3}\b").unwrap();

        for sect in file.section_headers() {
            if let Ok(bytes) = file.get_section_bytes(sect) {
                for s in config.clone().enumerate(sect.VirtualAddress, bytes) {
                    let string = std::str::from_utf8(s.string).unwrap();
                    if url_regex.is_match(string) || ip_regex.is_match(string) {
                        output += &format!(
                            "{}!{:?}:{:#x} {} {:?}\n",
                            self.file.file_name().unwrap().to_str().unwrap(),
                            sect.name(),
                            s.address,
                            if s.has_nul { "!" } else { "?" },
                            string
                        );
                    }
                }
            }
        }

        Ok(output)
    }
}

impl MyApp {
    #[cfg(feature = "scanner")]
    pub fn render_scanner(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        let modal_scanner = Modal::new(ctx, "scanner_dialog").with_close_on_outside_click(true);

        modal_scanner.show(|ui| {
            let path_buf = &mut self.ui.popups.scanner.dll;

            ui.label(if path_buf.is_empty() {
                "DLL:".to_string()
            } else {
                format!("DLL: {}", path_buf)
            });

            ui.add_space(5.0);

            if ui.cbutton("Browse").clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("DLL files", &["dll"])
                    .pick_file()
                {
                    *path_buf = path.to_string_lossy().into_owned();
                    if path_buf.ends_with(".dll") {
                        self.toasts.success("DLL selected.");
                    } else {
                        self.toasts.error("Please select a DLL file.");
                    }
                }
            }

            ui.add_space(5.0);

            if self.ui.popups.scanner.show_results {
                if ui.cbutton("Open results").clicked() {
                    match opener::open(self.app.meta.path.join("scanner_results.txt")) {
                        Ok(_) => {
                            self.toasts.success("Results opened.");
                        }
                        Err(err) => {
                            self.toasts
                                .error(format!("Failed to open results: {}", err));
                        }
                    }
                }

                ui.add_space(5.0);
            }

            ui.horizontal(|ui| {
                if ui.cbutton("Scan").clicked() {
                    if path_buf.is_empty() {
                        self.toasts.error("Please select a DLL file.");
                        return;
                    }

                    let scanner = Scanner::new(std::path::PathBuf::from(path_buf.clone()));

                    self.toasts
                        .info("Scanning PE-File using pelite...")
                        .duration(Some(std::time::Duration::from_secs(5)));

                    match scanner.scan(self.app.meta.path.clone()) {
                        Ok(()) => {
                            self.ui.popups.scanner.show_results = true;
                        }
                        Err(err) => {
                            self.toasts.error(err);
                        }
                    }
                }
                if ui.cbutton("Cancel").clicked() {
                    modal_scanner.close();
                }
            });
        });

        if ui.cbutton("Scanner").clicked() {
            modal_scanner.open();
        }

        ui.add_space(5.0);
    }
}
