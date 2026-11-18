use super::Server;
use std::{fs, net::TcpStream, time::Duration};
impl Server {
    pub(super) fn ready(&self) -> Result<(), String> {
        let address = self.url.trim_start_matches("http://");
        for _ in 0..100 {
            if TcpStream::connect(address).is_ok() {
                return Ok(());
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        Err("v2 server could not start".into())
    }
}
impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = fs::remove_dir_all(&self.dir);
    }
}
