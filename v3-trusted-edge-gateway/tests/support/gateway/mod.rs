mod extra;
mod start;
use crate::support::kit::StderrDrain;
use std::net::SocketAddr;
use std::process::Child;

pub struct Gateway {
    child: Child,
    stderr: StderrDrain,
    _dir: tempfile::TempDir,
    pub addr: SocketAddr,
}
impl Gateway {
    pub fn start(
        kernel_url: &str,
        token: [u8; 32],
        apps: serde_json::Value,
        max_body: usize,
    ) -> Self {
        start::start_env(
            kernel_url,
            token,
            apps,
            serde_json::json!({"max_body_bytes": max_body}),
            &[],
        )
    }
    pub fn start_with(
        kernel_url: &str,
        token: [u8; 32],
        apps: serde_json::Value,
        overrides: serde_json::Value,
    ) -> Self {
        start::start_env(kernel_url, token, apps, overrides, &[])
    }
    pub fn start_env(
        kernel_url: &str,
        token: [u8; 32],
        apps: serde_json::Value,
        overrides: serde_json::Value,
        envs: &[(&str, &str)],
    ) -> Self {
        start::start_env(kernel_url, token, apps, overrides, envs)
    }
}
