use serde::Deserialize;

pub struct Updater {
    pub current_version: String,
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
                    return Some(release.tag_name.trim_start_matches('v').to_string());
                }
            }
        }
        None
    }

    pub fn check_version(&mut self) -> bool {
        if let Some(remote_version) = self.get_remote_version() {
            match (
                semver::Version::parse(&self.current_version),
                semver::Version::parse(&remote_version),
            ) {
                (Ok(current), Ok(remote)) => {
                    if remote > current {
                        self.need_update = true;
                        return true;
                    } else {
                        return false;
                    }
                }
                _ => false,
            }
        } else {
            false
        }
    }
}

impl Default for Updater {
    fn default() -> Self {
        Self {
            current_version: env!("CARGO_PKG_VERSION").to_string(),
            repository: "AnarchyLoader/AnarchyLoader".to_string(),
            need_update: false,
        }
    }
}

#[derive(Deserialize)]
pub struct Release {
    tag_name: String,
    prerelease: bool,
}