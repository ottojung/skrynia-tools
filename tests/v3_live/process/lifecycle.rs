use super::Server;
use std::{
    fs,
    net::TcpStream,
    process::{Command, Stdio},
    time::Duration,
};
impl Server {
    pub fn ready(&mut self) -> Result<(), String> {
        let addr = self.url.trim_start_matches("http://").trim_end_matches('/');
        for _ in 0..300 {
            if TcpStream::connect(addr).is_ok() {
                return Ok(());
            }
            if let Some(status) = self.child.try_wait().map_err(|e| e.to_string())? {
                return Err(format!("server exited: {status}"));
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        Err("server did not start".into())
    }
    pub fn restart(&mut self) -> Result<(), String> {
        self.child.kill().map_err(|e| e.to_string())?;
        self.child.wait().map_err(|e| e.to_string())?;
        let addr = self.url.trim_start_matches("http://").trim_end_matches('/');
        self.child = Command::new(&self.bin)
            .env("SKRYNIA_BIND", addr)
            .env("SKRYNIA_ROOT_WASM", &self.wasm)
            .env("SKRYNIA_DATA_DIR", &self.dir)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| e.to_string())?;
        self.ready()
    }
}
impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = fs::remove_dir_all(&self.dir);
    }
}
