//! Spawns the real compiled trusted-edge gateway binary.
use std::fs;
use std::io;
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process::Child;
use std::{thread, time};

pub struct EdgeProc {
    child: Child,
    pub port: u16,
    pub host: String,
    pub origin: String,
    pub cap: String,
    _base: PathBuf,
}
impl EdgeProc {
    pub fn start(art: &Path, base: &Path, kernel_port: u16, token: &[u8; 32]) -> io::Result<Self> {
        let port = super::free_port();
        let host = "push-app.test".to_owned();
        let origin = "https://push-app.test".to_owned();
        let cap = "edge-capability-fullchain-0001".to_owned();
        fs::create_dir_all(base)?;
        super::edge_cfg::write(base, port, kernel_port, token, &host, &origin, &cap)?;
        let child = super::edge_cfg::spawn(art, base)?;
        let e = Self {
            child,
            port,
            host,
            origin,
            cap,
            _base: base.to_path_buf(),
        };
        e.wait_ready()?;
        Ok(e)
    }
    fn wait_ready(&self) -> io::Result<()> {
        for _ in 0..150 {
            if TcpStream::connect(("127.0.0.1", self.port)).is_ok() {
                return Ok(());
            }
            thread::sleep(time::Duration::from_millis(100));
        }
        Err(io::Error::other("edge not ready"))
    }
}
impl Drop for EdgeProc {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
