use std::collections::BTreeMap;

use super::config::Config;
use crate::{Hack, MyApp};

impl MyApp {
    pub fn group_hacks_by_game(&self) -> BTreeMap<String, BTreeMap<String, Vec<Hack>>> {
        let mut all_hacks = self.app.hacks.clone();
        all_hacks.extend(self.app.config.local_hacks.iter().map(|lh| {
            Hack {
                name: std::path::Path::new(&lh.dll)
                    .file_name()
                    .map(|os_str| os_str.to_string_lossy().to_string())
                    .unwrap_or_else(|| "Unknown".to_string()),
                process: lh.process.clone(),
                file: std::path::Path::new(&lh.dll)
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
                file_path: std::path::Path::new(&lh.dll).to_path_buf(),
                game: "Added".to_string(),
                local: true,
                arch: lh.arch.clone(),
                ..Default::default()
            }
        }));
        Self::group_hacks_by_game_internal(&all_hacks, &self.app.config)
    }

    pub fn group_hacks_by_game_internal(
        hacks: &[Hack],
        config: &Config,
    ) -> BTreeMap<String, BTreeMap<String, Vec<Hack>>> {
        let mut hacks_by_game: BTreeMap<String, BTreeMap<String, Vec<Hack>>> = BTreeMap::new();

        for hack in hacks {
            if config.show_only_favorites && !config.favorites.contains(&hack.name) {
                continue;
            }

            let game = hack.game.clone();

            if game.starts_with("CSS") {
                Self::group_css_hacks_internal(&mut hacks_by_game, hack.clone());
            } else if game.starts_with("Rust") {
                Self::group_rust_hacks_internal(&mut hacks_by_game, hack.clone());
            } else {
                Self::group_other_hacks_internal(&mut hacks_by_game, hack.clone());
            }
        }
        hacks_by_game
    }

    pub fn group_css_hacks_internal(
        hacks_by_game: &mut BTreeMap<String, BTreeMap<String, Vec<Hack>>>,
        hack: Hack,
    ) {
        let parts = hack.game.split_whitespace();
        let game_name = "CSS".to_string();
        let version = parts.skip(1).collect::<Vec<&str>>().join(" ");
        let version = if version.is_empty() {
            "Default".to_string()
        } else {
            version
        };
        hacks_by_game
            .entry(game_name)
            .or_insert_with(BTreeMap::new)
            .entry(version)
            .or_insert_with(Vec::new)
            .push(hack);
    }

    pub fn group_rust_hacks_internal(
        hacks_by_game: &mut BTreeMap<String, BTreeMap<String, Vec<Hack>>>,
        hack: Hack,
    ) {
        let parts = hack.game.split(",");
        let game_name = "Rust (NonSteam)".to_string();
        let version = parts.skip(1).collect::<Vec<&str>>().join(",");
        let version = if version.is_empty() {
            "Default".to_string()
        } else {
            version
        };

        hacks_by_game
            .entry(game_name)
            .or_insert_with(BTreeMap::new)
            .entry(version)
            .or_insert_with(Vec::new)
            .push(hack);
    }

    pub fn group_other_hacks_internal(
        hacks_by_game: &mut BTreeMap<String, BTreeMap<String, Vec<Hack>>>,
        hack: Hack,
    ) {
        hacks_by_game
            .entry(hack.game.clone())
            .or_insert_with(BTreeMap::new)
            .entry("".to_string())
            .or_insert_with(Vec::new)
            .push(hack);
    }
}
