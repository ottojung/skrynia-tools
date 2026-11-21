mod cfg;

use super::Gateway;
use crate::support::kit::{free_port, wait_ready};
use std::net::SocketAddr;
use std::process::{Command, Stdio};

pub fn start_with(
    kernel_url: &str,
    token: [u8; 32],
    apps: serde_json::Value,
    overrides: serde_json::Value,
) -> Gateway {
    start_env(kernel_url, token, apps, overrides, &[])
}
pub fn start_env(
    kernel_url: &str,
    token: [u8; 32],
    apps: serde_json::Value,
    overrides: serde_json::Value,
    envs: &[(&str, &str)],
) -> Gateway {
    let dir = tempfile::tempdir().unwrap();
    let token_path = dir.path().join("operator.token");
    std::fs::write(&token_path, token).unwrap();
    cfg::set_mode(&token_path, 0o600);
    let addr: SocketAddr = format!("127.0.0.1:{}", free_port()).parse().unwrap();
    let config = cfg::build(addr, kernel_url, &token_path, apps, overrides);
    let cfg_path = dir.path().join("config.json");
    std::fs::write(&cfg_path, serde_json::to_vec(&config).unwrap()).unwrap();
    cfg::set_mode(&cfg_path, 0o600);
    let bin = env!("CARGO_BIN_EXE_skrynia-v3-trusted-edge-gateway");
    let mut cmd = Command::new(bin);
    cmd.env("SKRYNIA_EDGE_GATEWAY_CONFIG", &cfg_path)
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    for (k, v) in envs {
        cmd.env(k, v);
    }
    let mut child = cmd.spawn().unwrap();
    wait_ready(addr, &mut child);
    Gateway {
        child,
        _dir: dir,
        addr,
    }
}
