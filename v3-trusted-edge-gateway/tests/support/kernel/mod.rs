mod mode;
mod start;
pub mod wasm;

use crate::support::kit::StderrDrain;
use std::net::SocketAddr;
use std::process::Child;

pub struct Kernel {
    child: Child,
    stderr: StderrDrain,
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

    pub fn stderr_text(&mut self) -> String {
        let _ = self.child.kill();
        let _ = self.child.wait();
        self.stderr.finish();
        self.stderr.snapshot()
    }
}

impl Drop for Kernel {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        self.stderr.finish();
        if std::thread::panicking() {
            let stderr = self.stderr.snapshot();
            if !stderr.trim().is_empty() {
                eprintln!("--- kernel stderr ({}) ---\n{stderr}", self.addr);
            }
        }
    }
}
