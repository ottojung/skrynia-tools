use super::Kernel;
use super::wasm;
use crate::support::kit::{free_port, wait_ready};
use std::net::SocketAddr;
use std::process::{Command, Stdio};

pub fn start(bin: &str, p0_wasm: &str, token: [u8; 32]) -> Kernel {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join("programs")).unwrap();
    std::fs::create_dir_all(dir.path().join("security")).unwrap();
    set_mode(dir.path().join("security"), 0o700);
    let digest = <sha2::Sha256 as sha2::Digest>::digest(token);
    let sec = dir.path().join("security/program0.sha256");
    std::fs::write(&sec, digest).unwrap();
    set_mode(&sec, 0o600);
    std::fs::write(dir.path().join("programs/root.wasm"), wasm::echo_wasm()).unwrap();
    let addr: SocketAddr = format!("127.0.0.1:{}", free_port()).parse().unwrap();
    let mut child = Command::new(bin)
        .env("SKRYNIA_ROOT_WASM", p0_wasm)
        .env("SKRYNIA_BIND", addr.to_string())
        .env("SKRYNIA_DATA_DIR", dir.path())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    wait_ready(addr, &mut child);
    Kernel {
        child,
        _dir: dir,
        addr,
    }
}

fn set_mode(path: impl AsRef<std::path::Path>, mode: u32) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode)).unwrap();
    }
    #[cfg(not(unix))]
    {
        let _ = (path, mode);
    }
}
