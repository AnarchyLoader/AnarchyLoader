use serde::{Deserialize, Serialize};

use crate::MyApp;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LocalHack {
    pub dll: String,
    pub name: String,
    pub process: String,
    pub arch: String,
}

impl LocalHack {
    pub fn new(dll: String, process: String, arch: String) -> Self {
        let name = std::path::Path::new(&dll)
            .file_stem()
            .map(|f| f.to_string_lossy().into_owned())
            .unwrap_or_default();

        Self {
            dll,
            name,
            process,
            arch,
        }
    }
}

#[derive(Debug, Default)]
pub struct LocalUI {
    pub(crate) new_local_dll: String,
    pub(crate) new_local_process: String,
    pub(crate) new_local_arch: String,
}

impl MyApp {
    pub fn add_local_hack(&mut self, hack: LocalHack) {
        self.app.config.local_hacks.push(hack);
        self.app.config.save();
    }
}
