//! Spawns the real compiled trusted-edge gateway binary.
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::{fs, io, thread, time};

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
        write_cfg(base, port, kernel_port, token, &host, &origin, &cap)?;
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
        e.wait_ready()?;
        Ok(e)
    }
    fn wait_ready(&self) -> io::Result<()> {
        for _ in 0..150 {
            if std::net::TcpStream::connect(("127.0.0.1", self.port)).is_ok() {
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
fn write_cfg(
    base: &Path,
    port: u16,
    kport: u16,
    token: &[u8; 32],
    host: &str,
    origin: &str,
    cap: &str,
) -> io::Result<()> {
    let tok = base.join("operator.token");
    fs::write(&tok, token)?;
    chmod600(&tok)?;
    let cfg = serde_json::json!({
        "bind": format!("127.0.0.1:{port}"),
        "upstream": format!("http://127.0.0.1:{kport}/"),
        "operator_token_file": tok.to_string_lossy(),
        "apps": [{"name":"push","hosts":[host],"origins":[origin],"edge_capabilities":[cap]}]
    });
    let p = base.join("edge.json");
    fs::write(&p, serde_json::to_vec(&cfg).map_err(io::Error::other)?)?;
    chmod600(&p)
}
#[cfg(unix)]
fn chmod600(p: &Path) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(p, fs::Permissions::from_mode(0o600))
}
#[cfg(not(unix))]
fn chmod600(_: &Path) -> io::Result<()> {
    Ok(())
}
