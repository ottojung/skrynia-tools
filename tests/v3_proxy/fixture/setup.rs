use std::{fs, path::PathBuf};
pub fn prepare(guest: Option<&[u8]>) -> PathBuf {
    let temp = std::env::temp_dir().join(format!(
        "skrynia-dumb-proxy-{}-{}",
        std::process::id(),
        rand::random::<u64>()
    ));
    fs::create_dir_all(temp.join("programs")).unwrap();
    if let Some(code) = guest {
        fs::write(temp.join("programs/root.wasm"), code).unwrap();
    }
    temp
}
