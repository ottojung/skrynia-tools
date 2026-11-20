use super::*;
pub fn startup_rejected(guest: &[u8]) -> bool {
    let bin = std::env::var("SKRYNIA_V3_SERVER_BIN").unwrap();
    let root = std::env::var("SKRYNIA_V3_PROGRAM0_WASM").unwrap();
    let temp = setup::prepare(Some(guest));
    let mut cmd = Command::new(bin);
    cmd.env("SKRYNIA_ROOT_WASM", root)
        .env("SKRYNIA_PROGRAM1_WASM", temp.join("program1.wasm"))
        .env("SKRYNIA_DATA_DIR", &temp)
        .env("SKRYNIA_BIND", "127.0.0.1:0")
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    let status = cmd.status().unwrap();
    let _ = std::fs::remove_dir_all(temp);
    !status.success()
}
