use std::io::Read;
use std::net::TcpStream;

pub(super) fn read_prefix(s: &mut TcpStream) -> Vec<u8> {
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
    seen
}
