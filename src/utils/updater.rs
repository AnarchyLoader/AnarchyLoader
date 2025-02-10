use serde::Deserialize;

#[derive(Debug)]
pub struct Updater {
    pub current_version: String,
    pub new_version: Option<String>,
    pub repository: String,
    pub need_update: bool,
}

impl Updater {
    pub fn get_latest_releases(&self) -> Result<Vec<Release>, ureq::Error> {
        let url = format!("https://api.github.com/repos/{}/releases", self.repository);
        let response: Vec<Release> = ureq::get(&url).call()?.into_json()?;
        Ok(response)
    }

    pub fn get_remote_version(&self) -> Option<String> {
        if let Ok(releases) = self.get_latest_releases() {
            for release in releases {
                if !release.prerelease {
                    log::info!("<UPDATER> Found remote version: {}", release.tag_name);
                    return Some(release.tag_name.trim_start_matches('v').to_string());
                }
            }
        }
        log::info!("<UPDATER> No suitable remote version found");
        None
    }

    pub fn check_version(&mut self) -> Result<bool, String> {
        log::info!("<UPDATER> Checking version");
        if let Some(remote_version) = self.get_remote_version() {
            match (
                semver::Version::parse(&self.current_version),
                semver::Version::parse(&remote_version),
            ) {
                (Ok(current), Ok(remote)) => {
                    if remote > current {
                        log::info!(
                            "<UPDATER> Update needed: current version {} < remote version {}",
                            current,
                            remote
                        );
                        self.new_version = Some(remote_version);
                        self.need_update = true;
                        Ok(true)
                    } else {
                        Ok(false)
                    }
                }
                _ => {
                    log::error!("<UPDATER> Failed to parse versions");
                    Err("Failed to parse versions".to_string())
                }
            }
        } else {
            log::error!("<UPDATER> Failed to get remote version");
            Err("Failed to get remote version".to_string())
        }
    }
}

impl Default for Updater {
    fn default() -> Self {
        Self {
            current_version: env!("CARGO_PKG_VERSION").to_string(),
            repository: "AnarchyLoader/AnarchyLoader".to_string(),
            need_update: false,
            new_version: None,
        }
    }
}

#[derive(Deserialize)]
pub struct Release {
    tag_name: String,
    prerelease: bool,
}
