use super::Server;
use std::{io, net::TcpStream, thread, time::Duration};

impl Server {
    pub(super) fn wait_ready(&mut self) -> io::Result<()> {
        for _ in 0..200 {
            if TcpStream::connect(("127.0.0.1", self.port)).is_ok() {
                return Ok(());
            }
            if let Some(st) = self.child.try_wait()? {
                return Err(io::Error::other(format!("server exited: {st}")));
            }
            thread::sleep(Duration::from_millis(25));
        }
        Err(io::Error::other("server did not become ready"))
    }
    pub(super) fn stop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
impl Drop for Server {
    fn drop(&mut self) {
        self.stop();
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}
