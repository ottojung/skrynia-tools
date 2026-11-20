mod cleanup;
mod restart;
mod setup;
mod token;
use std::{
    net::TcpListener,
    process::{Child, Command, Stdio},
    time::Duration,
};
pub struct Server {
    pub url: String,
    pub child: Child,
    dir: std::path::PathBuf,
}
impl Server {
    pub fn start(guest: Option<&[u8]>) -> Self {
        let bin = std::env::var("SKRYNIA_V3_SERVER_BIN").unwrap();
        let root = std::env::var("SKRYNIA_V3_PROGRAM0_WASM").unwrap();
        let temp = setup::prepare(guest);
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        drop(listener);
        let child = Command::new(bin)
            .env("SKRYNIA_ROOT_WASM", root)
            .env("SKRYNIA_BIND", address.to_string())
            .env("SKRYNIA_DATA_DIR", &temp)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let mut out = Self {
            url: format!("http://{address}/"),
            child,
            dir: temp,
        };
        for _ in 0..500 {
            if std::net::TcpStream::connect(address).is_ok() {
                return out;
            }
            assert!(out.child.try_wait().unwrap().is_none(), "kernel terminated");
            std::thread::sleep(Duration::from_millis(20));
        }
        panic!("proxy kernel did not start");
    }
}
