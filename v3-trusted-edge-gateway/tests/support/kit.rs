use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::Path;
use std::process::Child;
use std::time::Duration;

pub fn kit() -> Option<(String, String)> {
    let bin = std::env::var("SKRYNIA_V3_SERVER_BIN").ok();
    let p0 = std::env::var("SKRYNIA_V3_PROGRAM0_WASM").ok();
    if bin.is_none() && p0.is_none() && std::env::var_os("SKRYNIA_REQUIRE_KIT").is_none() {
        return None;
    }
    let bin = bin.expect("kernel artifact required");
    let p0 = p0.expect("root WASM artifact required");
    assert!(
        Path::new(&bin).is_file() && Path::new(&p0).is_file(),
        "missing kit artifact"
    );
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
