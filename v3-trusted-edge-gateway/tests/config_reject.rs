mod support;
use std::process::{Command, Stdio};
use std::time::Duration;
use support::kit::free_port;

#[test]
fn rejects_duplicate_edge_bearer_across_apps() {
    let dir = tempfile::tempdir().unwrap();
    let token_path = dir.path().join("operator.token");
    std::fs::write(&token_path, [0x42u8; 32]).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&token_path, std::fs::Permissions::from_mode(0o600)).unwrap();
    }
    let addr = format!("127.0.0.1:{}", free_port());
    let cfg = serde_json::json!({
        "bind": addr, "upstream": "http://127.0.0.1:1/",
        "operator_token_file": token_path.to_string_lossy(), "max_body_bytes": 4096,
        "apps": [
            {"name":"a","hosts":["long.a.vau.place"],"origins":["https://long.a.vau.place"],"edge_capabilities":["same-cap"]},
            {"name":"b","hosts":["kawun.a.vau.place"],"origins":["https://kawun.a.vau.place"],"edge_capabilities":["same-cap"]}
        ]
    });
    let cfg_path = dir.path().join("config.json");
    std::fs::write(&cfg_path, serde_json::to_vec(&cfg).unwrap()).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&cfg_path, std::fs::Permissions::from_mode(0o600)).unwrap();
    }
    let status = Command::new(env!("CARGO_BIN_EXE_skrynia-v3-trusted-edge-gateway"))
        .env("SKRYNIA_EDGE_GATEWAY_CONFIG", &cfg_path)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .unwrap();
    assert!(!status.success(), "duplicate edge bearer must be rejected");
    let _ = Duration::from_millis(0);
}
