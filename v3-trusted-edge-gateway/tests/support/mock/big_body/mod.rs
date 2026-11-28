mod request;
mod serve;

pub(super) use super::dechunk;

use std::net::{SocketAddr, TcpListener};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::thread;
use std::time::Duration;

pub fn big_body_server(
    total: usize,
    operator: [u8; 32],
) -> (SocketAddr, Arc<AtomicBool>, Arc<AtomicUsize>) {
    let l = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = l.local_addr().unwrap();
    let stop = Arc::new(AtomicBool::new(false));
    let written = Arc::new(AtomicUsize::new(0));
    let f = stop.clone();
    let w = written.clone();
    thread::spawn(move || {
        l.set_nonblocking(true).ok();
        while !f.load(Ordering::Relaxed) {
            if let Ok((s, _)) = l.accept() {
                let sf = f.clone();
                let sw = w.clone();
                thread::spawn(move || serve::serve(s, sf, sw, total, operator));
            }
            thread::sleep(Duration::from_millis(10));
        }
    });
    (addr, stop, written)
}
