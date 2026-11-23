//! Spawns the real compiled kernel PR125 with dumb P0 and push P1 WASM.
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::{fs, io, thread, time};

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
        prepare(base, art, &token)?;
        let log = fs::File::create(base.join("kernel.log"))?;
        let child = spawn(art, base, port, log)?;
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
            if std::net::TcpStream::connect(("127.0.0.1", self.port)).is_ok() {
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
    sha256(b"fullchain-operator-cap-v1")
}
fn prepare(base: &Path, art: &Path, token: &[u8; 32]) -> io::Result<()> {
    if base.exists() {
        fs::remove_dir_all(base)?;
    }
    let op = base.join("operator");
    let us = base.join("user");
    fs::create_dir_all(op.join("security"))?;
    fs::create_dir_all(op.join("programs"))?;
    for s in ["active", "blob", "ready", "appdata"] {
        fs::create_dir_all(us.join("programs").join(s))?;
    }
    fs::write(op.join("security/program0.sha256"), sha256(token))?;
    fs::write(
        op.join("programs/root.wasm"),
        fs::read(art.join("p1_push_efe6117.wasm"))?,
    )?;
    fs::write(us.join("programs/operator.cap"), op_cap())?;
    Ok(())
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
    use sha2::{Digest, Sha256};
    Sha256::digest(b).into()
}
