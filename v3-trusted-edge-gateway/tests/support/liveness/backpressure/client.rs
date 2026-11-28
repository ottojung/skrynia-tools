use std::io::{ErrorKind, Read};
use std::net::TcpStream;
use std::time::Duration;

use super::{TOTAL, dechunk};

const READ_TIMEOUT: Duration = Duration::from_secs(6);

pub(super) fn assert_terminated(c1: &mut TcpStream, prefix: Vec<u8>) {
    c1.set_read_timeout(Some(READ_TIMEOUT)).unwrap();
    let mut tail = Vec::new();
    let read = c1.read_to_end(&mut tail);
    assert!(
        read.is_ok() || matches!(&read, Err(e) if e.kind() == ErrorKind::ConnectionReset),
        "client1 must be terminated by the gateway, not by a test read timeout: {read:?}"
    );
    let mut full = prefix;
    full.extend_from_slice(&tail);
    let sep = full
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .expect("client1 response headers");
    let head = std::str::from_utf8(&full[..sep]).expect("client1 headers utf8");
    assert!(
        head.starts_with("HTTP/1.1 200"),
        "client1 must receive the upstream status: {head:?}"
    );
    assert!(
        head.to_ascii_lowercase()
            .contains("transfer-encoding: chunked"),
        "client1 response must be chunked: {head:?}"
    );
    assert!(
        full.len() - (sep + 4) < TOTAL,
        "client1 must not receive the full body"
    );
    assert!(
        dechunk(&full[sep + 4..]).is_err(),
        "deadline must not complete a partial HTTP response"
    );
}
