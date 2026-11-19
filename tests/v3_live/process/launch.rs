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
        let stderr_path = dir.join("server.stderr.log");
        let stderr_file = fs::File::create(&stderr_path).map_err(|e| e.to_string())?;
        let stderr_file_child = stderr_file.try_clone().map_err(|e| e.to_string())?;
        let mut server = Self {
            url: format!("http://127.0.0.1:{}/", addr.port()),
            child: Command::new(&bin)
                .env("SKRYNIA_BIND", addr.to_string())
                .env("SKRYNIA_ROOT_WASM", &wasm)
                .env("SKRYNIA_DATA_DIR", &dir)
                .stdout(Stdio::null())
                .stderr(Stdio::from(stderr_file_child))
                .spawn()
                .map_err(|e| e.to_string())?,
            dir,
            bin,
            wasm,
            stderr_path,
        };
        drop(stderr_file);
        server.ready()?;
        Ok(server)
    }
}
