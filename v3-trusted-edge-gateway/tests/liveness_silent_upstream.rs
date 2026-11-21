mod support;
use support::{apps::two_apps, gateway::Gateway, http::post_raw, mock};

#[test]
fn silent_upstream_times_out_and_recovers() {
    let (up, stop) = mock::hang_server();
    let token = [0x42u8; 32];
    let cap = "edge-cap-hang";
    let gw = Gateway::start_with(
        &format!("http://{up}/"),
        token,
        two_apps(cap, "edge-cap-b"),
        serde_json::json!({
            "upstream_read_timeout_ms": 300,
            "request_timeout_ms": 800,
            "acquire_timeout_ms": 1000
        }),
    );
    let (st, _, _) = post_raw(
        gw.addr,
        "long.a.vau.place",
        Some("https://long.a.vau.place"),
        Some(cap),
        b"x",
    );
    assert!(
        st == 504 || st == 502 || st >= 500,
        "must fail closed: {st}"
    );
    let (st2, _, _) = post_raw(
        gw.addr,
        "long.a.vau.place",
        Some("https://long.a.vau.place"),
        Some(cap),
        b"y",
    );
    assert!(
        st2 == 504 || st2 == 502 || st2 >= 500,
        "must recover: {st2}"
    );
    stop.store(true, std::sync::atomic::Ordering::Relaxed);
}
