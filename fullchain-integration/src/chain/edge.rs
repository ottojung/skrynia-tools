//! Spawns the real compiled trusted-edge gateway binary.
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::{fs, io};

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
        super::verify_all(art)?;
        let port = super::free_port();
        let host = "push-app.test".to_owned();
        let origin = "https://push-app.test".to_owned();
        let cap = "edge-capability-fullchain-0001".to_owned();
        fs::create_dir_all(base)?;
        super::edge_cfg::write_cfg(base, port, kernel_port, token, &host, &origin, &cap)?;
        let log = fs::File::create(base.join("edge.log"))?;
        let child = Command::new(art.join("trusted_edge_9fa4df14"))
            .env("SKRYNIA_EDGE_GATEWAY_CONFIG", base.join("edge.json"))
            .stdout(Stdio::from(log.try_clone()?))
            .stderr(Stdio::from(log))
            .spawn()?;
        let e = Self {
            child,
            port,
            host,
            origin,
            cap,
            _base: base.to_path_buf(),
        };
        super::proc_ready::wait(e.port, "edge")?;
        Ok(e)
    }
}

impl Drop for EdgeProc {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
