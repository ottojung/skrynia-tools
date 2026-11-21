use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

pub fn trickle(mut s: TcpStream, stop: Arc<AtomicBool>, delay_ms: u64) {
    let _ = s.set_read_timeout(Some(Duration::from_secs(2)));
    let mut buf = [0u8; 4096];
    let _ = s.read(&mut buf);
    let hdr = b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n";
    if s.write_all(hdr).is_err() {
        return;
    }
    while !stop.load(Ordering::Relaxed) {
        if s.write_all(b"1\r\na\r\n").is_err() {
            break;
        }
        std::thread::sleep(Duration::from_millis(delay_ms));
    }
}

pub fn ok(mut s: TcpStream) {
    let _ = s.set_read_timeout(Some(Duration::from_secs(2)));
    let mut buf = [0u8; 4096];
    let _ = s.read(&mut buf);
    let _ = s.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok");
}
