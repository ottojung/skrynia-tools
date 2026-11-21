use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::thread;
use std::time::Duration;

pub fn big_body_server(total: usize) -> (SocketAddr, Arc<AtomicBool>, Arc<AtomicUsize>) {
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
                thread::spawn(move || serve(s, sf, sw, total));
            }
            thread::sleep(Duration::from_millis(10));
        }
    });
    (addr, stop, written)
}
fn serve(mut s: TcpStream, stop: Arc<AtomicBool>, written: Arc<AtomicUsize>, total: usize) {
    let _ = s.set_read_timeout(Some(Duration::from_secs(5)));
    let _ = s.set_write_timeout(Some(Duration::from_secs(10)));
    let mut buf = [0u8; 8192];
    let _ = s.read(&mut buf);
    let hdr = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {total}\r\nConnection: close\r\n\r\n"
    );
    if s.write_all(hdr.as_bytes()).is_err() {
        return;
    }
    let chunk = vec![b'x'; 64 * 1024];
    let mut sent = 0usize;
    while sent < total && !stop.load(Ordering::Relaxed) {
        let n = std::cmp::min(chunk.len(), total - sent);
        if s.write_all(&chunk[..n]).is_err() {
            break;
        }
        sent += n;
        written.store(sent, Ordering::Relaxed);
    }
}
