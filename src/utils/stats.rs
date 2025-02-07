use std::{collections::HashMap, fs, path::PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Statistics {
    pub opened_count: u64,
    pub inject_counts: HashMap<String, u64>,
    pub total_seconds: u64,
}

impl Default for Statistics {
    fn default() -> Self {
        Statistics {
            opened_count: 0,
            inject_counts: HashMap::new(),
            total_seconds: 0,
        }
    }
}

pub fn calculate_session(time: String) -> String {
    let session_start = chrono::DateTime::parse_from_rfc3339(&time)
        .unwrap()
        .with_timezone(&chrono::Local);
    let session_duration = chrono::Local::now() - session_start;
    let hours = session_duration.num_hours();
    let minutes = session_duration.num_minutes() % 60;
    let seconds = session_duration.num_seconds() % 60;
    if hours > 0 {
        format!("{} hours and {} minutes", hours, minutes)
    } else if minutes > 0 {
        format!("{} minutes", minutes)
    } else {
        format!("{} seconds", seconds)
    }
}

pub fn get_time_from_seconds(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let seconds = seconds % 60;
    if hours > 0 {
        format!(
            "{} hours, {} minutes and {} seconds",
            hours, minutes, seconds
        )
    } else if minutes > 0 {
        format!("{} minutes and {} seconds", minutes, seconds)
    } else {
        format!("{} seconds", seconds)
    }
}

pub fn get_time_difference_in_seconds(time: chrono::DateTime<chrono::FixedOffset>) -> u64 {
    let current_time =
        chrono::Local::now().with_timezone(&chrono::FixedOffset::east_opt(0).unwrap());
    let time_difference = current_time - time;
    time_difference.num_seconds() as u64
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

    pub fn increment_total_time(&mut self, time: u64) {
        self.total_seconds += time;
        self.save();
    }

    pub fn has_injections(&self) -> bool {
        !self.inject_counts.is_empty()
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
