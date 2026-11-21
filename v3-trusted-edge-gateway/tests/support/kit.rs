use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::Path;
use std::process::Child;
use std::time::Duration;

pub fn kit() -> Option<(String, String)> {
    let bin = std::env::var("SKRYNIA_V3_SERVER_BIN").ok()?;
    let p0 = std::env::var("SKRYNIA_V3_PROGRAM0_WASM").ok()?;
    if !Path::new(&bin).exists() || !Path::new(&p0).exists() {
        return None;
    }
    Some((bin, p0))
}

pub fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

pub fn wait_ready(addr: SocketAddr, child: &mut Child) {
    for _ in 0..500 {
        if TcpStream::connect(addr).is_ok() {
            return;
        }
        assert!(child.try_wait().unwrap().is_none(), "process exited early");
        std::thread::sleep(Duration::from_millis(20));
    }
    panic!("did not become ready: {addr}");
}
