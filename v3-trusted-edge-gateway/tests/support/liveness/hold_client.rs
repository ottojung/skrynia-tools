use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

pub fn send_and_hold_headers(
    addr: SocketAddr,
    host: &str,
    origin: &str,
    cap: &str,
    body: &[u8],
) -> TcpStream {
    let mut s = TcpStream::connect(addr).unwrap();
    #[cfg(unix)]
    unsafe {
        use std::os::unix::io::AsRawFd;
        let buf: libc::c_int = 4096;
        libc::setsockopt(
            s.as_raw_fd(),
            libc::SOL_SOCKET,
            libc::SO_RCVBUF,
            &buf as *const _ as *const libc::c_void,
            std::mem::size_of_val(&buf) as libc::socklen_t,
        );
    }
    s.set_read_timeout(Some(Duration::from_secs(5))).ok();
    s.set_write_timeout(Some(Duration::from_secs(5))).ok();
    let req = format!(
        "POST /_skrynia/ HTTP/1.1\r\nHost: {host}\r\nOrigin: {origin}\r\nX-Skrynia-Edge-Capability: {cap}\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    s.write_all(req.as_bytes()).unwrap();
    s.write_all(body).unwrap();
    let mut buf = [0u8; 4096];
    let mut seen = Vec::new();
    loop {
        let n = s.read(&mut buf).unwrap_or(0);
        if n == 0 {
            break;
        }
        seen.extend_from_slice(&buf[..n]);
        if seen.windows(4).any(|w| w == b"\r\n\r\n") {
            break;
        }
    }
    s
}
