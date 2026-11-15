use super::server;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::{fs, process::Command};

fn command(parts: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_skrynia-tools"))
        .args(parts)
        .output()
        .unwrap()
}
pub fn verify() {
    let root = std::env::temp_dir().join(format!("skrynia-resume-{:032x}", rand::random::<u128>()));
    fs::create_dir(&root).unwrap();
    let file = root.join("creds.json");
    let path = file.to_str().unwrap();
    let grant = serde_json::json!({"id":"test123","issuerToken":"issuer-top-secret"});
    let initial = serde_json::json!({"state":"grant_saved","grant":grant});
    fs::write(&file, serde_json::to_vec(&initial).unwrap()).unwrap();
    #[cfg(unix)]
    fs::set_permissions(&file, fs::Permissions::from_mode(0o600)).unwrap();
    let inspect = command(&["admin", "inspect", "--credentials-file", path]);
    let printed = String::from_utf8(inspect.stdout).unwrap();
    assert!(inspect.status.success());
    assert!(printed.contains("grant_saved"));
    assert!(!printed.contains("issuer-top-secret"));
    let (url, worker) = server::once();
    let finished = command(&["admin", "resume", "--credentials-file", path, "--url", &url]);
    assert!(
        finished.status.success(),
        "{}",
        String::from_utf8_lossy(&finished.stderr)
    );
    worker.join().unwrap();
    assert!(!String::from_utf8_lossy(&finished.stdout).contains("issued-very-secret-user"));
    let saved: serde_json::Value = serde_json::from_slice(&fs::read(&file).unwrap()).unwrap();
    assert_eq!(saved["user"]["userToken"], "issued-very-secret-user");
    let second = command(&["admin", "resume", "--credentials-file", path]);
    assert!(second.status.success());
    fs::remove_dir_all(root).unwrap();
}
