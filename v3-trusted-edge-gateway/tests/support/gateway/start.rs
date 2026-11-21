use super::Gateway;
use crate::support::kit::{free_port, wait_ready};
use std::net::SocketAddr;
use std::process::{Command, Stdio};

pub fn start(
    kernel_url: &str,
    token: [u8; 32],
    apps: serde_json::Value,
    max_body: usize,
) -> Gateway {
    let dir = tempfile::tempdir().unwrap();
    let token_path = dir.path().join("operator.token");
    std::fs::write(&token_path, token).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&token_path, std::fs::Permissions::from_mode(0o600)).unwrap();
    }
    let addr: SocketAddr = format!("127.0.0.1:{}", free_port()).parse().unwrap();
    let cfg = serde_json::json!({"bind":addr.to_string(),"upstream":kernel_url,"operator_token_file":token_path.to_string_lossy(),"max_body_bytes":max_body,"apps":apps});
    let cfg_path = dir.path().join("config.json");
    std::fs::write(&cfg_path, serde_json::to_vec(&cfg).unwrap()).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&cfg_path, std::fs::Permissions::from_mode(0o600)).unwrap();
    }
    let bin = env!("CARGO_BIN_EXE_skrynia-v3-trusted-edge-gateway");
    let mut child = Command::new(bin)
        .env("SKRYNIA_EDGE_GATEWAY_CONFIG", &cfg_path)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    wait_ready(addr, &mut child);
    Gateway {
        child,
        _dir: dir,
        addr,
    }
}
