use super::hold_client::send_and_hold_headers;
use super::{apps::two_apps, gateway::Gateway, http::post_raw, mock};
use std::thread;
use std::time::Duration;

pub fn response_hold_saturates_then_recovers() {
    let (up, stop, _) = mock::trickle_first_then_ok(200);
    let token = [0x42u8; 32];
    let cap = "edge-cap-rh";
    let gw = Gateway::start_with(
        &format!("http://{up}/"),
        token,
        two_apps(cap, "edge-cap-b"),
        serde_json::json!({
            "max_concurrent_requests": 1,
            "acquire_timeout_ms": 300,
            "upstream_read_timeout_ms": 5000,
            "request_timeout_ms": 8000
        }),
    );
    let h = send_and_hold_headers(
        gw.addr,
        "long.a.vau.place",
        "https://long.a.vau.place",
        cap,
        b"a",
    );
    thread::sleep(Duration::from_millis(200));
    let (stb, _, _) = req(gw.addr, cap, b"b");
    assert_eq!(stb, 503, "held response must consume permit");
    drop(h);
    thread::sleep(Duration::from_millis(300));
    let (stc, _, body) = req(gw.addr, cap, b"c");
    assert_eq!(stc, 200);
    assert_eq!(body, b"ok");
    stop.store(true, std::sync::atomic::Ordering::Relaxed);
}

fn req(
    addr: std::net::SocketAddr,
    cap: &str,
    body: &[u8],
) -> (u16, Vec<(String, String)>, Vec<u8>) {
    post_raw(
        addr,
        "long.a.vau.place",
        Some("https://long.a.vau.place"),
        Some(cap),
        body,
    )
}
