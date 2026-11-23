//! Spawns the real compiled kernel PR125 with dumb P0 and push P1 WASM.
use std::fs;
use std::io;
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process::Child;
use std::{thread, time};

pub struct Kernel {
    child: Child,
    pub port: u16,
    pub token: [u8; 32],
    pub base: PathBuf,
}

impl Kernel {
    pub fn start(art: &Path, base: &Path) -> io::Result<Self> {
        let port = super::free_port();
        let token = *b"fullchain-disposable-token-00001";
        super::kernel_prepare::prepare(base, art, &token)?;
        let log = fs::File::create(base.join("kernel.log"))?;
        let child = super::kernel_prepare::spawn(art, base, port, log)?;
        let k = Self {
            child,
            port,
            token,
            base: base.to_path_buf(),
        };
        k.wait_ready()?;
        Ok(k)
    }
    fn wait_ready(&self) -> io::Result<()> {
        for _ in 0..150 {
            if TcpStream::connect(("127.0.0.1", self.port)).is_ok() {
                return Ok(());
            }
            thread::sleep(time::Duration::from_millis(100));
        }
        Err(io::Error::other("kernel not ready"))
    }
}
impl Drop for Kernel {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
pub fn op_cap() -> [u8; 32] {
    super::kernel_prepare::sha256(b"fullchain-operator-cap-v1")
}
