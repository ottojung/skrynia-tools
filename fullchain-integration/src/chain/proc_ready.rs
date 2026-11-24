//! Loopback readiness probe shared by the disposable kernel and edge processes.
use std::io;
use std::{thread, time};

pub(super) fn wait(port: u16, label: &str) -> io::Result<()> {
    for _ in 0..150 {
        if std::net::TcpStream::connect(("127.0.0.1", port)).is_ok() {
            return Ok(());
        }
        thread::sleep(time::Duration::from_millis(100));
    }
    Err(io::Error::other(format!("{label} not ready")))
}
