use super::Server;
use std::{
    fs,
    net::TcpListener,
    process::{Command, Stdio},
};
impl Server {
    pub fn new(bin: String, wasm: String) -> Result<Self, String> {
        let sock = TcpListener::bind("127.0.0.1:0").map_err(|e| e.to_string())?;
        let addr = sock.local_addr().map_err(|e| e.to_string())?;
        drop(sock);
        let dir = std::env::temp_dir().join(format!("skrynia-tools-live-{}", std::process::id()));
        if dir.exists() {
            return Err("live fixture already exists".into());
        }
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let mut server = Self {
            url: format!("http://127.0.0.1:{}/", addr.port()),
            child: Command::new(&bin)
                .env("SKRYNIA_BIND", addr.to_string())
                .env("SKRYNIA_ROOT_WASM", &wasm)
                .env("SKRYNIA_DATA_DIR", &dir)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .map_err(|e| e.to_string())?,
            dir,
            bin,
            wasm,
        };
        server.ready()?;
        Ok(server)
    }
}
