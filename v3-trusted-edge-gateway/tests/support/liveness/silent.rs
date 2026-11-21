use super::{apps::two_apps, gateway::Gateway, http::post_raw, mock};

pub fn silent_upstream_times_out_and_recovers() {
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
    let (st, _, _) = req(gw.addr, cap, b"x");
    assert!(
        st == 504 || st == 502 || st >= 500,
        "must fail closed: {st}"
    );
    let (st2, _, _) = req(gw.addr, cap, b"y");
    assert!(
        st2 == 504 || st2 == 502 || st2 >= 500,
        "must recover: {st2}"
    );
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
