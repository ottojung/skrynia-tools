mod stall;
use super::hold_client::send_and_hold_headers_small_rcvbuf;
use super::{apps::two_apps, gateway::Gateway, http::post_raw, mock};
use std::io::{ErrorKind, Read};
use std::net::{SocketAddr, TcpStream};
use std::sync::atomic::Ordering;
use std::thread;
use std::time::{Duration, Instant};

const TOTAL: usize = 16 * 1024 * 1024;
const OPERATOR: [u8; 32] = [0x42u8; 32];
const REQUEST_TIMEOUT_MS: u64 = 2000;
const READ_TIMEOUT: Duration = Duration::from_secs(6);

pub fn backpressure_nonreading_downstream() {
    let (up, stop, written) = mock::big_body_server(TOTAL, OPERATOR);
    let cap = "edge-cap-bp";
    let gw = start_gateway(up, cap);
    let deadline = Instant::now() + Duration::from_millis(REQUEST_TIMEOUT_MS);
    let (mut c1, prefix) = send_and_hold_headers_small_rcvbuf(
        gw.addr,
        "long.a.vau.place",
        "https://long.a.vau.place",
        cap,
        b"a",
    );
    stall::wait_backpressure(&written, TOTAL, deadline);

    let (held, _, _) = req(gw.addr, cap);
    assert_eq!(held, 503, "A: before deadline held permit must reject");
    thread::sleep(Duration::from_millis(2800));

    let (recovered, _, body) = req(gw.addr, cap);
    assert_eq!(recovered, 200, "B: after deadline must recover");
    assert_eq!(
        body,
        b"ok".to_vec(),
        "B: recovery must use the small independent upstream response"
    );
    assert_client_terminated(&mut c1, prefix);

    drop(c1);
    thread::sleep(Duration::from_millis(100));
    let (dropped, _, body) = req(gw.addr, cap);
    assert_eq!(dropped, 200, "C: explicit drop recovers");
    assert_eq!(
        body,
        b"ok".to_vec(),
        "C: recovery must use the small independent upstream response"
    );
    stop.store(true, Ordering::Relaxed);
}

fn start_gateway(up: SocketAddr, cap: &str) -> Gateway {
    Gateway::start_env(
        &format!("http://{up}/"),
        OPERATOR,
        two_apps(cap, "edge-cap-b2"),
        serde_json::json!({
            "max_concurrent_requests": 1,
            "request_timeout_ms": REQUEST_TIMEOUT_MS,
            "acquire_timeout_ms": 250,
            "max_body_bytes": 20 * 1024 * 1024,
        }),
        &[],
    )
}

fn assert_client_terminated(c1: &mut TcpStream, prefix: Vec<u8>) {
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
        super::http::dechunk(&full[sep + 4..]).is_err(),
        "deadline must not complete a partial HTTP response"
    );
}

fn req(addr: SocketAddr, cap: &str) -> (u16, Vec<(String, String)>, Vec<u8>) {
    post_raw(
        addr,
        "long.a.vau.place",
        Some("https://long.a.vau.place"),
        Some(cap),
        b"b",
    )
}
