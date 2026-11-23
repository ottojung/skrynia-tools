//! Writes the trusted-edge config and spawns the gateway process.
use std::fs;
use std::io;
use std::path::Path;
use std::process::{Child, Command, Stdio};

pub fn write(
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
pub fn spawn(art: &Path, base: &Path) -> io::Result<Child> {
    let log = fs::File::create(base.join("edge.log"))?;
    Command::new(art.join("trusted_edge_9fa4df14"))
        .env("SKRYNIA_EDGE_GATEWAY_CONFIG", base.join("edge.json"))
        .stdout(Stdio::from(log.try_clone()?))
        .stderr(Stdio::from(log))
        .spawn()
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
