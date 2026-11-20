use std::{fs, path::Path};
pub fn data(dir: &Path, guest: &str) {
    fs::create_dir_all(dir.join("data")).unwrap();
    fs::copy(guest, dir.join("data/v2.wasm")).unwrap();
    fs::write(dir.join("data/seed_public.txt"), b"abcdef").unwrap();
    let root = wat::parse_str(include_str!("../root.wat")).unwrap();
    fs::write(dir.join("root.wasm"), root).unwrap();
}
