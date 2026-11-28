use std::io::Write;
use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

use super::prefix::read_prefix;
use super::sockopt::set_small_rcvbuf;

pub(super) fn send_and_hold(
    addr: SocketAddr,
    host: &str,
    origin: &str,
    cap: &str,
    body: &[u8],
    small_rcvbuf: bool,
) -> (TcpStream, Vec<u8>) {
    let mut s = TcpStream::connect(addr).unwrap();
    if small_rcvbuf {
        set_small_rcvbuf(&s);
    }
    s.set_read_timeout(Some(Duration::from_secs(5))).ok();
    s.set_write_timeout(Some(Duration::from_secs(5))).ok();
    let req = format!(
        "POST /_skrynia/ HTTP/1.1\r\nHost: {host}\r\nOrigin: {origin}\r\nX-Skrynia-Edge-Capability: {cap}\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    s.write_all(req.as_bytes()).unwrap();
    s.write_all(body).unwrap();
    let prefix = read_prefix(&mut s);
    (s, prefix)
}
