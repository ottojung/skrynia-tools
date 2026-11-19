mod launch;
mod life;
use std::{
    io,
    path::{Path, PathBuf},
    process::Child,
};

pub struct Server {
    bin: String,
    wasm: String,
    child: Child,
    dir: PathBuf,
    port: u16,
    pub url: String,
}
impl Server {
    pub fn new(bin: String, wasm: String) -> io::Result<Self> {
        let port = launch::reserve_port()?;
        let dir = std::env::temp_dir().join(format!("skrynia-p0-v2auth-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir)?;
        let child = launch::spawn(&bin, &wasm, &dir, port)?;
        let mut s = Self {
            bin,
            wasm,
            child,
            dir,
            port,
            url: format!("http://127.0.0.1:{port}/"),
        };
        s.wait_ready()?;
        Ok(s)
    }
    pub fn restart(&mut self) -> io::Result<()> {
        self.stop();
        self.child = launch::spawn(&self.bin, &self.wasm, &self.dir, self.port)?;
        self.wait_ready()
    }
    pub fn data_dir(&self) -> &Path {
        &self.dir
    }
}
