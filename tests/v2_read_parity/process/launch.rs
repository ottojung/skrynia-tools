use super::Server;
use std::{
    fs,
    net::TcpListener,
    process::{Command, Stdio},
};
impl Server {
    pub fn start(bin: &str) -> Result<Self, String> {
        let listener = TcpListener::bind("127.0.0.1:0").map_err(|e| e.to_string())?;
        let port = listener.local_addr().map_err(|e| e.to_string())?.port();
        drop(listener);
        let dir = std::env::temp_dir().join(format!("skrynia-v2-parity-{}", std::process::id()));
        if dir.exists() {
            return Err("temporary v2 fixture exists".into());
        }
        fs::create_dir_all(dir.join("store")).map_err(|e| e.to_string())?;
        fs::write(dir.join("store/a.txt"), b"abcdef").map_err(|e| e.to_string())?;
        let url = format!("http://127.0.0.1:{port}");
        let child = Command::new(bin)
            .env("SKRYNIA_HOST", "127.0.0.1")
            .env("SKRYNIA_PORT", port.to_string())
            .env("SKRYNIA_URL", &url)
            .env("SKRYNIA_DATA_DIR", &dir)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| e.to_string())?;
        let server = Self { url, child, dir };
        server.ready()?;
        Ok(server)
    }
}
