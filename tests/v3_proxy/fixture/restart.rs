use super::Server;
use std::{
    net::TcpStream,
    process::{Command, Stdio},
    time::Duration,
};
impl Server {
    pub fn restart_with_guest(&mut self, wasm: &[u8]) {
        self.child.kill().unwrap();
        self.child.wait().unwrap();
        std::fs::write(self.dir.join("programs/root.wasm"), wasm).unwrap();
        let host = self
            .url
            .strip_prefix("http://")
            .unwrap()
            .strip_suffix('/')
            .unwrap();
        self.child = Command::new(std::env::var("SKRYNIA_V3_SERVER_BIN").unwrap())
            .env(
                "SKRYNIA_ROOT_WASM",
                std::env::var("SKRYNIA_V3_PROGRAM0_WASM").unwrap(),
            )
            .env("SKRYNIA_DATA_DIR", &self.dir)
            .env("SKRYNIA_BIND", host)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        for _ in 0..500 {
            if TcpStream::connect(host).is_ok() {
                return;
            }
            assert!(self.child.try_wait().unwrap().is_none(), "restart crashed");
            std::thread::sleep(Duration::from_millis(20));
        }
        panic!("kernel did not restart");
    }
}
