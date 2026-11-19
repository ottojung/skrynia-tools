use std::{
    io,
    net::TcpListener,
    path::Path,
    process::{Child, Command, Stdio},
};

pub fn reserve_port() -> io::Result<u16> {
    Ok(TcpListener::bind(("127.0.0.1", 0))?.local_addr()?.port())
}
pub fn spawn(bin: &str, wasm: &str, dir: &Path, port: u16) -> io::Result<Child> {
    Command::new(bin)
        .env("SKRYNIA_BIND", format!("127.0.0.1:{port}"))
        .env("SKRYNIA_ROOT_WASM", wasm)
        .env("SKRYNIA_DATA_DIR", dir)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .spawn()
}
