use std::path::PathBuf;

use egui_modal::Modal;
#[cfg(feature = "scanner")]
use pelite::{
    pe64::{Pe, PeFile},
    FileMap,
};

use crate::{utils::custom_widgets::Button, MyApp};

pub struct Scanner {
    pub file: PathBuf,
}

#[derive(Clone)]
pub struct ScannerPopup {
    pub dll: String,
    pub show_results: bool,
}

#[cfg(feature = "scanner")]
impl Scanner {
    pub fn new(file: PathBuf) -> Scanner {
        Scanner { file }
    }

    pub fn scan(&self, app_path: PathBuf) -> Result<(), String> {
        let mut output = String::new();

        let file_map = FileMap::open(self.file.as_path()).unwrap();

        let file = match PeFile::from_bytes(file_map.as_ref()) {
            Ok(file) => file,
            Err(err) => return Err(format!("Failed to parse PE file: {}", err)),
        };

        let imports = match file.imports() {
            Ok(imports) => imports,
            Err(err) => return Err(format!("Failed to read imports: {}", err)),
        };

        output += "Imports:\n";

        for desc in imports {
            let dll_name = desc.dll_name();
            let iat = match desc.iat() {
                Ok(iat) => iat,
                Err(err) => {
                    return Err(format!(
                        "Failed to read imports for {:?}: {}",
                        dll_name, err
                    ))
                }
            };
            output += &format!(
                "Imported {} functions from {}\n",
                iat.len(),
                dll_name.unwrap()
            );
        }

        output += "====================\n\n";

        std::fs::write(app_path.join("scanner_results.txt"), output).unwrap();

        Ok(())
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
                    match opener::open(self.app_path.join("scanner_results.txt")) {
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

                    match scanner.scan(self.app_path.clone()) {
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
