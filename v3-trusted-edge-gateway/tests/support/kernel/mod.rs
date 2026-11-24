mod start;
pub mod wasm;

use std::net::SocketAddr;
use std::process::Child;

pub struct Kernel {
    child: Child,
    _dir: tempfile::TempDir,
    pub addr: SocketAddr,
}

impl Kernel {
    pub fn start(bin: &str, p0_wasm: &str, token: [u8; 32]) -> Self {
        start::start(bin, p0_wasm, token)
    }

    pub fn url(&self) -> String {
        format!("http://{}/", self.addr)
    }
}

impl Drop for Kernel {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
