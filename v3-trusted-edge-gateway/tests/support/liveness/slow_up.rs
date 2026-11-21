use super::{apps::two_apps, gateway::Gateway, http::post_raw, kernel::Kernel, kit::kit, slow};
use std::time::Duration;

pub fn slow_unfinished_chunked_recovers() {
    let Some((bin, p0)) = kit() else {
        eprintln!("SKIP: canonical kit required");
        return;
    };
    let token = [0x42u8; 32];
    let cap = "edge-cap-slow";
    let kernel = Kernel::start(&bin, &p0, token);
    let gw = Gateway::start_with(
        &kernel.url(),
        token,
        two_apps(cap, "edge-cap-b"),
        serde_json::json!({"upload_idle_timeout_ms": 400, "request_timeout_ms": 3000}),
    );
    let s = slow::start_unfinished_chunked(
        gw.addr,
        "long.a.vau.place",
        "https://long.a.vau.place",
        cap,
    );
    std::thread::sleep(Duration::from_millis(100));
    let st = slow::read_status(s);
    assert!(st == 0 || st >= 400, "slow upload must fail closed: {st}");
    let (st2, _, body) = post_raw(
        gw.addr,
        "long.a.vau.place",
        Some("https://long.a.vau.place"),
        Some(cap),
        b"recover",
    );
    assert_eq!(st2, 200);
    assert_eq!(body, b"recover");
}
