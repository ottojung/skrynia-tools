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
        let program1 = std::env::var("SKRYNIA_V3_PROGRAM1_WASM")
            .map_err(|e| format!("SKRYNIA_V3_PROGRAM1_WASM: {e}"))?;
        fs::create_dir_all(dir.join("kernel")).map_err(|e| e.to_string())?;
        fs::create_dir_all(dir.join("service/programs")).map_err(|e| e.to_string())?;
        let private = dir.join("service/programs/operator.cap");
        fs::write(&private, [85u8; 32]).map_err(|e| e.to_string())?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&private, fs::Permissions::from_mode(0o600))
                .map_err(|e| e.to_string())?;
        }
        let mut server = Self {
            url: format!("http://127.0.0.1:{}/", addr.port()),
            child: Command::new(&bin)
                .env("SKRYNIA_BIND", addr.to_string())
                .env("SKRYNIA_ROOT_WASM", &wasm)
                .env("SKRYNIA_DATA_DIR", dir.join("kernel"))
                .env("SKRYNIA_PROGRAM1_WASM", &program1)
                .env("SKRYNIA_PROGRAM1_DATA_DIR", dir.join("service"))
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .map_err(|e| e.to_string())?,
            dir,
            bin,
            wasm,
            program1,
        };
        server.ready()?;
        Ok(server)
    }
}
