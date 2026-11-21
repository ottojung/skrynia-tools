use super::{apps::two_apps, gateway::Gateway, http::post_raw, mock};
use std::thread;
use std::time::Duration;

pub fn saturated_slots_then_benign_responsive() {
    let (up, stop, _count) = mock::hang_first_then_hello();
    let token = [0x42u8; 32];
    let cap = "edge-cap-sat";
    let gw = Gateway::start_with(
        &format!("http://{up}/"),
        token,
        two_apps(cap, "edge-cap-b"),
        serde_json::json!({
            "max_concurrent_requests": 1,
            "acquire_timeout_ms": 300,
            "upstream_read_timeout_ms": 1200,
            "request_timeout_ms": 4000
        }),
    );
    let addr = gw.addr;
    let h = thread::spawn(move || req(addr, cap, b"a").0);
    thread::sleep(Duration::from_millis(150));
    assert_eq!(req(gw.addr, cap, b"b").0, 503, "saturated slot must reject");
    let sta = h.join().unwrap();
    assert!(
        sta == 504 || sta == 502 || sta >= 500,
        "holder must fail closed"
    );
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
