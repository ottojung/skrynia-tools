use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

pub fn post_chunked_overflow(
    addr: SocketAddr,
    host: &str,
    origin: &str,
    cap: &str,
    chunk: &[u8],
    count: usize,
) -> (u16, Vec<u8>) {
    let mut s = TcpStream::connect(addr).unwrap();
    s.set_read_timeout(Some(Duration::from_secs(10))).ok();
    s.set_write_timeout(Some(Duration::from_secs(10))).ok();
    let req = format!(
        "POST /_skrynia/ HTTP/1.1\r\nHost: {host}\r\nOrigin: {origin}\r\nX-Skrynia-Edge-Capability: {cap}\r\nContent-Type: application/octet-stream\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n"
    );
    let _ = s.write_all(req.as_bytes());
    for _ in 0..count {
        let frame = format!("{:x}\r\n", chunk.len());
        if s.write_all(frame.as_bytes()).is_err() {
            break;
        }
        if s.write_all(chunk).is_err() {
            break;
        }
        if s.write_all(b"\r\n").is_err() {
            break;
        }
    }
    let _ = s.write_all(b"0\r\n\r\n");
    let mut resp = Vec::new();
    let _ = s.read_to_end(&mut resp);
    if resp.is_empty() {
        return (0, resp);
    }
    if let Some(sep) = resp.windows(4).position(|w| w == b"\r\n\r\n") {
        let head = std::str::from_utf8(&resp[..sep]).unwrap_or("");
        let status = head
            .lines()
            .next()
            .and_then(|l| l.split_whitespace().nth(1))
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        return (status, resp);
    }
    (0, resp)
}
