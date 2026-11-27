use std::net::{SocketAddr, TcpStream};
use std::process::Child;
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub type StderrBuf = Arc<Mutex<Vec<u8>>>;

pub fn stderr_snapshot(buf: &StderrBuf) -> String {
    let g = buf.lock().unwrap_or_else(|e| e.into_inner());
    String::from_utf8_lossy(&g).into_owned()
}

pub fn wait_ready(addr: SocketAddr, child: &mut Child, stderr: &StderrBuf) {
    for _ in 0..500 {
        if TcpStream::connect(addr).is_ok() {
            return;
        }
        if let Some(status) = child.try_wait().unwrap() {
            panic!("process exited early: {status}\nstderr:\n{}", stderr_snapshot(stderr));
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    let _ = child.kill();
    let _ = child.wait();
    panic!("did not become ready: {addr}\nstderr:\n{}", stderr_snapshot(stderr));
}
