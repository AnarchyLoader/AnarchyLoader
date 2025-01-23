use std::{collections::HashMap, fs, path::PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Statistics {
    pub opened_count: u64,
    pub inject_counts: HashMap<String, u64>,
}

impl Default for Statistics {
    fn default() -> Self {
        Statistics {
            opened_count: 0,
            inject_counts: HashMap::new(),
        }
    }
}

impl Statistics {
    pub fn increment_inject_count(&mut self, hack_name: &str) {
        let count = self.inject_counts.entry(hack_name.to_string()).or_insert(0);
        *count += 1;
        self.save();
    }

    pub fn increment_opened_count(&mut self) {
        self.opened_count += 1;
        self.save();
    }

    pub fn load() -> Self {
        let statistics_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("anarchyloader");

        fs::create_dir_all(&statistics_dir).ok();
        let statistics_path = statistics_dir.join("statistics.json");

        if let Ok(data) = fs::read_to_string(&statistics_path) {
            serde_json::from_str::<Statistics>(&data).unwrap_or_default()
        } else {
            Statistics::default()
        }
    }

    pub fn save(&self) {
        let statistics_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("anarchyloader");

        fs::create_dir_all(&statistics_dir).ok();
        let statistics_path = statistics_dir.join("statistics.json");

        if let Ok(data) = serde_json::to_string_pretty(&self) {
            fs::write(statistics_path, data).ok();
        }
    }

    pub fn reset(&mut self) {
        *self = Statistics::default();
        self.save();
    }
}
