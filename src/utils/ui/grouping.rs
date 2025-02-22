use std::collections::BTreeMap;

use crate::{utils::config::Config, Hack, MyApp};
impl MyApp {
    pub fn get_all_hacks(hacks: &[Hack], config: &Config) -> Vec<Hack> {
        let mut all_hacks = Vec::with_capacity(hacks.len() + config.local_hacks.len());
        all_hacks.extend_from_slice(hacks);

        all_hacks.extend(config.local_hacks.iter().map(|lh| {
            let file_path = std::path::Path::new(&lh.dll);

            let name = std::path::Path::new(&lh.dll)
                .file_stem()
                .map(|f| f.to_string_lossy().into_owned())
                .unwrap_or_default();

            Hack {
                name,
                process: lh.process.clone(),
                file: file_path
                    .file_name()
                    .map(|f| f.to_string_lossy().into_owned())
                    .unwrap_or_default(),
                file_path: file_path.to_path_buf(),
                game: "Added".to_string(),
                local: true,
                arch: lh.arch.clone(),
                ..Default::default()
            }
        }));

        all_hacks
    }

    pub fn group_hacks_by_game(
        hacks: &[Hack],
        config: &Config,
    ) -> BTreeMap<String, BTreeMap<String, Vec<Hack>>> {
        let grouped_hacks =
            Self::group_hacks_by_game_internal(&Self::get_all_hacks(hacks, config), config);

        grouped_hacks
    }

    pub(crate) fn group_hacks_by_game_internal(
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
            .or_default()
            .entry(version)
            .or_default()
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
            .or_default()
            .entry(version)
            .or_default()
            .push(hack);
    }

    pub fn group_other_hacks_internal(
        hacks_by_game: &mut BTreeMap<String, BTreeMap<String, Vec<Hack>>>,
        hack: Hack,
    ) {
        hacks_by_game
            .entry(hack.game.clone())
            .or_default()
            .entry("".to_string())
            .or_default()
            .push(hack);
    }
}
