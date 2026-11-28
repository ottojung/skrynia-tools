use std::io::Write;
use std::net::TcpStream;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Duration;

use super::request::read_request;

pub(super) fn serve(
    mut s: TcpStream,
    stop: Arc<AtomicBool>,
    written: Arc<AtomicUsize>,
    total: usize,
    operator: [u8; 32],
) {
    let _ = s.set_write_timeout(Some(Duration::from_secs(10)));
    if read_request(&mut s, operator) {
        let body = b"ok";
        let hdr = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            body.len()
        );
        s.write_all(hdr.as_bytes()).unwrap();
        s.write_all(body).unwrap();
        return;
    }
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
