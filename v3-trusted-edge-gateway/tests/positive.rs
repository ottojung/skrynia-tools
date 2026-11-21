mod support;
use support::{apps::two_apps, gateway::Gateway, http::post_raw, kernel::Kernel, kit::kit};

#[test]
fn positive_echo() {
    let Some((bin, p0)) = kit() else {
        eprintln!("SKIP: canonical kit required");
        return;
    };
    let token = [0x42u8; 32];
    let cap = "edge-cap-a";
    let kernel = Kernel::start(&bin, &p0, token);
    let gw = Gateway::start(
        &kernel.url(),
        token,
        two_apps(cap, "edge-cap-b"),
        16 * 1024 * 1024,
    );
    let (st, _, body) = post_raw(
        gw.addr,
        "long.a.vau.place",
        Some("https://long.a.vau.place"),
        Some(cap),
        b"hello-opaque",
    );
    assert_eq!(st, 200, "positive");
    assert_eq!(body, b"hello-opaque");
}
