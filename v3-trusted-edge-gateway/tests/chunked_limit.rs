mod support;
use std::time::Duration;
use support::{
    apps::two_apps,
    gateway::Gateway,
    http::{post_chunked_overflow, post_raw},
    kernel::Kernel,
    kit::kit,
};

#[test]
fn chunked_no_content_length_over_limit_fails_closed_and_live() {
    let Some((bin, p0)) = kit() else {
        eprintln!("SKIP: canonical kit required");
        return;
    };
    let token = [0x42u8; 32];
    let cap = "edge-cap-limit";
    let kernel = Kernel::start(&bin, &p0, token);
    let gw = Gateway::start(&kernel.url(), token, two_apps(cap, "edge-cap-other"), 4096);
    let chunk = vec![b'a'; 1024];
    let (st, _) = post_chunked_overflow(
        gw.addr,
        "long.a.vau.place",
        "https://long.a.vau.place",
        cap,
        &chunk,
        16,
    );
    assert!(
        st == 413 || st == 400 || st == 502 || st == 0 || st >= 500,
        "over-limit must fail closed"
    );
    std::thread::sleep(Duration::from_millis(150));
    let (st2, _, body) = post_raw(
        gw.addr,
        "long.a.vau.place",
        Some("https://long.a.vau.place"),
        Some(cap),
        b"ok",
    );
    assert_eq!(st2, 200, "gateway remains live");
    assert_eq!(body, b"ok");
}
