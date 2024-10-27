#[cfg(target_os = "windows")]
fn main() {
    if let Err(e) = winres::WindowsResource::new()
        .set_icon("resources\\img\\icon.ico")
        .compile()
    {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

#[cfg(not(target_os = "windows"))]
fn main() {}
