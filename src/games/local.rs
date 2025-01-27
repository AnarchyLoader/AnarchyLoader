use serde::{Deserialize, Serialize};

use crate::MyApp;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LocalHack {
    pub dll: String,
    pub process: String,
    pub arch: String,
}

#[derive(Debug)]
pub struct LocalUI {
    pub(crate) new_local_dll: String,
    pub(crate) new_local_process: String,
    pub(crate) new_local_arch: String,
}

impl Default for LocalUI {
    fn default() -> Self {
        Self {
            new_local_dll: String::new(),
            new_local_process: String::new(),
            new_local_arch: String::new(),
        }
    }
}

impl MyApp {
    pub fn add_local_hack(&mut self, hack: LocalHack) {
        self.app.config.local_hacks.push(hack);
        self.app.config.save();
    }
}
