mod start;
use std::io::Read;
use std::net::SocketAddr;
use std::process::Child;

pub struct Gateway {
    child: Child,
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
        start::start_with(
            kernel_url,
            token,
            apps,
            serde_json::json!({"max_body_bytes": max_body}),
        )
    }
    pub fn start_with(
        kernel_url: &str,
        token: [u8; 32],
        apps: serde_json::Value,
        overrides: serde_json::Value,
    ) -> Self {
        start::start_with(kernel_url, token, apps, overrides)
    }
    pub fn stderr_text(&mut self) -> String {
        let _ = self.child.kill();
        let _ = self.child.wait();
        let mut s = String::new();
        if let Some(mut e) = self.child.stderr.take() {
            let _ = e.read_to_string(&mut s);
        }
        s
    }
}
impl Drop for Gateway {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
