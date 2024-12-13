#[cfg(target_os = "windows")]
extern crate embed_resource;

fn main() {
    let _ = embed_resource::compile("resources.rc", embed_resource::NONE);
}

#[cfg(not(target_os = "windows"))]
fn main() {}
