//! Spawns the real compiled kernel PR125 with dumb P0 and push P1 WASM.
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::{fs, io};

pub struct Kernel {
    child: Child,
    pub port: u16,
    pub token: [u8; 32],
    pub base: PathBuf,
}

impl Kernel {
    pub fn start(art: &Path, base: &Path) -> io::Result<Self> {
        super::verify_all(art)?;
        let port = super::free_port();
        let token = *b"fullchain-disposable-token-00001";
        super::kernel_fs::prepare(base, art, &token)?;
        let log = fs::File::create(base.join("kernel.log"))?;
        let child = spawn(art, base, port, log)?;
        let k = Self {
            child,
            port,
            token,
            base: base.to_path_buf(),
        };
        super::proc_ready::wait(k.port, "kernel")?;
        Ok(k)
    }
}

impl Drop for Kernel {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

pub fn op_cap() -> [u8; 32] {
    sha256(b"fullchain-operator-cap-v1")
}

fn spawn(art: &Path, base: &Path, port: u16, log: fs::File) -> io::Result<Child> {
    Command::new(art.join("kernel_PR125_856c029"))
        .env("SKRYNIA_ROOT_WASM", art.join("p0_d57ab27a.wasm"))
        .env("SKRYNIA_OPERATOR_DIR", base.join("operator"))
        .env("SKRYNIA_USER_DIR", base.join("user"))
        .env("SKRYNIA_BIND", format!("127.0.0.1:{port}"))
        .stdout(Stdio::from(log.try_clone()?))
        .stderr(Stdio::from(log))
        .spawn()
}

fn sha256(b: &[u8]) -> [u8; 32] {
    Sha256::digest(b).into()
}
