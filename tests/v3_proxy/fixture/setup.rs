use std::{fs, path::PathBuf};
pub fn prepare(guest: Option<&[u8]>) -> PathBuf {
    let temp = std::env::temp_dir().join(format!(
        "skrynia-dumb-proxy-{}-{}",
        std::process::id(),
        rand::random::<u64>()
    ));
    fs::create_dir_all(&temp).unwrap();
    if let Some(code) = guest {
        fs::write(temp.join("program1.wasm"), code).unwrap();
    }
    temp
}
