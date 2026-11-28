use std::io::Read;
use std::net::TcpStream;
use std::time::Duration;

use super::dechunk;

/// True only for the exact framed recovery request: 0x01 || operator || "b".
pub(super) fn read_request(s: &mut TcpStream, operator: [u8; 32]) -> bool {
    let _ = s.set_read_timeout(Some(Duration::from_secs(5)));
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
    body == recovery
}
