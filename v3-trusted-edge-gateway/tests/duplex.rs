mod support;
use support::{apps::two_apps, gateway::Gateway, http::post_raw, kernel::Kernel, kit::kit};

#[test]
fn full_duplex_13mib() {
    let Some((bin, p0)) = kit() else {
        eprintln!("SKIP: canonical kit required");
        return;
    };
    let token = [0x42u8; 32];
    let cap = "edge-cap-duplex";
    let kernel = Kernel::start(&bin, &p0, token);
    let gw = Gateway::start(
        &kernel.url(),
        token,
        two_apps(cap, "edge-cap-other"),
        16 * 1024 * 1024,
    );
    let size = 13 * 1024 * 1024;
    let mut payload = vec![0u8; size];
    for (i, b) in payload.iter_mut().enumerate() {
        *b = (i % 251) as u8;
    }
    let (st, _, body) = post_raw(
        gw.addr,
        "long.a.vau.place",
        Some("https://long.a.vau.place"),
        Some(cap),
        &payload,
    );
    assert_eq!(st, 200);
    assert_eq!(body.len(), payload.len());
    assert_eq!(body, payload);
}
