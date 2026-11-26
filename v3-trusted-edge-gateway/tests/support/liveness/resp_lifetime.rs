use super::{apps::two_apps, gateway::Gateway, mock};
use std::io::Read;
use std::net::TcpStream;
use std::time::Duration;

pub fn trickle_exceeds_absolute_lifetime() {
    let (up, stop, _) = mock::trickle_server(200);
    let token = [0x42u8; 32];
    let cap = "edge-cap-life";
    let gw = Gateway::start_with(
        &format!("http://{up}/"),
        token,
        two_apps(cap, "edge-cap-b"),
        serde_json::json!({
            "upstream_read_timeout_ms": 5000,
            "request_timeout_ms": 800,
            "acquire_timeout_ms": 1000
        }),
    );
    let mut s = TcpStream::connect(gw.addr).unwrap();
    s.set_read_timeout(Some(Duration::from_secs(5))).ok();
    s.set_write_timeout(Some(Duration::from_secs(5))).ok();
    let req = format!(
        "POST /_skrynia/ HTTP/1.1\r\nHost: long.a.vau.place\r\nOrigin: https://long.a.vau.place\r\nX-Skrynia-Edge-Capability: {cap}\r\nContent-Type: application/octet-stream\r\nContent-Length: 1\r\nConnection: close\r\n\r\na"
    );
    std::io::Write::write_all(&mut s, req.as_bytes()).unwrap();
    let mut buf = Vec::new();
    let begin = std::time::Instant::now();
    let rc = s.read_to_end(&mut buf);
    assert!(
        begin.elapsed() < Duration::from_secs(4),
        "gateway must terminate before client timeout"
    );
    assert!(!buf.is_empty(), "should get response headers/body start");
    let sep = buf
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .expect("HTTP headers");
    let head = std::str::from_utf8(&buf[..sep]).unwrap();
    assert!(head.starts_with("HTTP/1.1 200"));
    assert!(
        head.to_ascii_lowercase()
            .contains("transfer-encoding: chunked")
    );
    assert!(rc.is_ok() || rc.unwrap_err().kind() == std::io::ErrorKind::ConnectionReset);
    assert!(
        super::http::dechunk(&buf[sep + 4..]).is_err(),
        "deadline must not complete a partial HTTP response"
    );
    stop.store(true, std::sync::atomic::Ordering::Relaxed);
}
