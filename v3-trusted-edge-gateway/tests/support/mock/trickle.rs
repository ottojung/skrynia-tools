use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::thread;
use std::time::Duration;

pub fn trickle_server(chunk_delay_ms: u64) -> (SocketAddr, Arc<AtomicBool>, Arc<AtomicUsize>) {
    let l = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = l.local_addr().unwrap();
    let stop = Arc::new(AtomicBool::new(false));
    let conns = Arc::new(AtomicUsize::new(0));
    let f = stop.clone();
    let c = conns.clone();
    thread::spawn(move || {
        l.set_nonblocking(true).ok();
        while !f.load(Ordering::Relaxed) {
            if let Ok((s, _)) = l.accept() {
                c.fetch_add(1, Ordering::Relaxed);
                let sf = f.clone();
                thread::spawn(move || serve(s, sf, chunk_delay_ms));
            }
            thread::sleep(Duration::from_millis(10));
        }
    });
    (addr, stop, conns)
}

fn serve(mut s: TcpStream, stop: Arc<AtomicBool>, delay_ms: u64) {
    let _ = s.set_read_timeout(Some(Duration::from_secs(2)));
    let mut buf = [0u8; 4096];
    let _ = s.read(&mut buf);
    let hdr = b"HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n";
    if s.write_all(hdr).is_err() {
        return;
    }
    while !stop.load(Ordering::Relaxed) {
        if s.write_all(b"1\r\na\r\n").is_err() {
            break;
        }
        thread::sleep(Duration::from_millis(delay_ms));
    }
}
