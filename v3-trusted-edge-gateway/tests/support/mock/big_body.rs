use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::thread;
use std::time::Duration;

use super::super::http::dechunk;

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
                thread::spawn(move || serve(s, sf, sw, total, operator));
            }
            thread::sleep(Duration::from_millis(10));
        }
    });
    (addr, stop, written)
}

fn serve(
    mut s: TcpStream,
    stop: Arc<AtomicBool>,
    written: Arc<AtomicUsize>,
    total: usize,
    operator: [u8; 32],
) {
    let _ = s.set_read_timeout(Some(Duration::from_secs(5)));
    let _ = s.set_write_timeout(Some(Duration::from_secs(10)));

    // The gateway forwards each request as one chunked stream:
    // [0x01 || operator] || client body. Read the complete framed request
    // instead of guessing from one TCP read, which may split headers/chunks.
    let mut raw = Vec::new();
    let mut buf = [0u8; 8192];
    loop {
        let n = s.read(&mut buf).unwrap_or(0);
        if n == 0 {
            break;
        }
        raw.extend_from_slice(&buf[..n]);
        let Some(sep) = raw.windows(4).position(|w| w == b"\r\n\r\n") else {
            continue;
        };
        let head = String::from_utf8_lossy(&raw[..sep]).to_ascii_lowercase();
        assert!(
            head.lines()
                .any(|l| l.starts_with("transfer-encoding:") && l.contains("chunked")),
            "gateway must forward the upstream body as chunked"
        );
        if dechunk(&raw[sep + 4..]).is_ok() {
            break;
        }
    }
    let sep = raw
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .unwrap_or_else(|| panic!("upstream request headers incomplete: {} bytes", raw.len()));
    let body = dechunk(&raw[sep + 4..])
        .unwrap_or_else(|e| panic!("upstream request body incomplete: {e}"));
    let mut recovery = vec![0x01u8];
    recovery.extend_from_slice(&operator);
    recovery.push(b'b');

    if body == recovery {
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
