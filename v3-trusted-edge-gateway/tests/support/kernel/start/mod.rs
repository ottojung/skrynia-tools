use super::mode::set_mode;
use super::Kernel;
use super::wasm;
use crate::support::kit::{free_port, spawn_stderr_drain, wait_ready};
use std::net::SocketAddr;
use std::process::{Command, Stdio};

pub fn start(bin: &str, p0_wasm: &str, token: [u8; 32]) -> Kernel {
    let dir = tempfile::tempdir().unwrap();
    let operator = dir.path().join("operator");
    let user = dir.path().join("user");
    std::fs::create_dir_all(operator.join("programs")).unwrap();
    std::fs::create_dir_all(operator.join("security")).unwrap();
    std::fs::create_dir(&user).unwrap();
    set_mode(operator.join("security"), 0o700);
    let digest = <sha2::Sha256 as sha2::Digest>::digest(token);
    let sec = operator.join("security/program0.sha256");
    std::fs::write(&sec, digest).unwrap();
    set_mode(&sec, 0o600);
    std::fs::write(
        operator.join("programs/root.wasm"),
        wasm::p0_child_echo_wasm(),
    )
    .unwrap();
    let addr: SocketAddr = format!("127.0.0.1:{}", free_port()).parse().unwrap();
    let mut child = Command::new(bin)
        .env("SKRYNIA_ROOT_WASM", p0_wasm)
        .env("SKRYNIA_BIND", addr.to_string())
        .env("SKRYNIA_OPERATOR_DIR", &operator)
        .env("SKRYNIA_USER_DIR", &user)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let stderr = spawn_stderr_drain(&mut child);
    wait_ready(addr, &mut child, &stderr.buf);
    Kernel {
        child,
        stderr,
        _dir: dir,
        addr,
    }
}
