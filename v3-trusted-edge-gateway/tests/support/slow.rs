use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::thread::JoinHandle;
use std::time::Duration;

pub fn start_unfinished_chunked(
    addr: SocketAddr,
    host: &str,
    origin: &str,
    cap: &str,
) -> TcpStream {
    let mut s = TcpStream::connect(addr).unwrap();
    s.set_read_timeout(Some(Duration::from_secs(10))).ok();
    s.set_write_timeout(Some(Duration::from_secs(5))).ok();
    let req = format!(
        "POST /_skrynia/ HTTP/1.1\r\nHost: {host}\r\nOrigin: {origin}\r\nX-Skrynia-Edge-Capability: {cap}\r\nContent-Type: application/octet-stream\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n"
    );
    s.write_all(req.as_bytes()).unwrap();
    s
}

pub fn read_status(mut s: TcpStream) -> u16 {
    let mut resp = Vec::new();
    let _ = s.set_read_timeout(Some(Duration::from_secs(10)));
    let _ = s.read_to_end(&mut resp);
    if resp.is_empty() {
        return 0;
    }
    if let Some(sep) = resp.windows(4).position(|w| w == b"\r\n\r\n") {
        let head = std::str::from_utf8(&resp[..sep]).unwrap_or("");
        return head
            .lines()
            .next()
            .and_then(|l| l.split_whitespace().nth(1))
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
    }
    0
}

pub fn spawn_unfinished_chunked(
    addr: SocketAddr,
    host: &'static str,
    origin: &'static str,
    cap: &'static str,
) -> JoinHandle<u16> {
    std::thread::spawn(move || read_status(start_unfinished_chunked(addr, host, origin, cap)))
}
