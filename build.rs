use std::process::Command;
#[cfg(target_os = "windows")]
use std::{env, io};

fn main() -> io::Result<()> {
    if env::var_os("CARGO_CFG_WINDOWS").is_some() {
        let mut res = winres::WindowsResource::new();
        let version = env::var("CARGO_PKG_VERSION").unwrap();
        let version_numbers: Vec<u16> = version.split('.').map(|v| v.parse().unwrap()).collect();
        let version_info = (version_numbers[0] as u64) << 48
            | (version_numbers[1] as u64) << 32
            | (version_numbers[2] as u64) << 16;

        res.set_icon("resources/img/icon.ico")
            .set_language(0x0409) // US English
            .set_version_info(winres::VersionInfo::PRODUCTVERSION, version_info);
        res.compile()?;

        // get commit
        let output = Command::new("git").args(&["rev-parse", "HEAD"]).output()?;
        let git_hash = String::from_utf8(output.stdout).unwrap();
        println!("cargo:rustc-env=GIT_HASH={}", git_hash);
    }
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn main() {}
