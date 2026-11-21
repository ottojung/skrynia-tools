use std::net::{SocketAddr, TcpListener};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

pub fn hang_server() -> (SocketAddr, Arc<AtomicBool>) {
    let l = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = l.local_addr().unwrap();
    let stop = Arc::new(AtomicBool::new(false));
    let flag = stop.clone();
    thread::spawn(move || {
        l.set_nonblocking(true).ok();
        let mut peers = Vec::new();
        while !flag.load(Ordering::Relaxed) {
            if let Ok((s, _)) = l.accept() {
                s.set_nonblocking(false).ok();
                peers.push(s);
            }
            thread::sleep(Duration::from_millis(10));
        }
        drop(peers);
    });
    (addr, stop)
}
