use std::net::SocketAddr;

use super::{Gateway, OPERATOR, REQUEST_TIMEOUT_MS, post_raw, two_apps};

pub(super) fn start(up: SocketAddr, cap: &str) -> Gateway {
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

pub(super) fn assert_recovery(addr: SocketAddr, cap: &str, label: &str) {
    let (status, _, body) = request(addr, cap);
    assert_eq!(status, 200, "{label}");
    assert_eq!(
        body,
        b"ok".to_vec(),
        "{label}: recovery must use the small independent upstream response"
    );
}

pub(super) fn request(addr: SocketAddr, cap: &str) -> (u16, Vec<(String, String)>, Vec<u8>) {
    post_raw(
        addr,
        "long.a.vau.place",
        Some("https://long.a.vau.place"),
        Some(cap),
        b"b",
    )
}
