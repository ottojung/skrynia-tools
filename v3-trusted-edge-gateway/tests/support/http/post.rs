use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

pub fn post_raw(
    addr: SocketAddr,
    host: &str,
    origin: Option<&str>,
    capability: Option<&str>,
    body: &[u8],
) -> (u16, Vec<(String, String)>, Vec<u8>) {
    let mut stream = TcpStream::connect(addr).unwrap();
    stream.set_read_timeout(Some(Duration::from_secs(120))).ok();
    stream
        .set_write_timeout(Some(Duration::from_secs(120)))
        .ok();
    let mut req = format!(
        "POST /_skrynia/ HTTP/1.1\r\nHost: {host}\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\nConnection: close\r\n",
        body.len()
    );
    if let Some(o) = origin {
        req.push_str(&format!("Origin: {o}\r\n"));
    }
    if let Some(c) = capability {
        req.push_str(&format!("X-Skrynia-Edge-Capability: {c}\r\n"));
    }
    req.push_str("\r\n");
    stream.write_all(req.as_bytes()).unwrap();
    stream.write_all(body).unwrap();
    let mut resp = Vec::new();
    let _ = stream.read_to_end(&mut resp);
    super::parse::parse_http(&resp)
}
