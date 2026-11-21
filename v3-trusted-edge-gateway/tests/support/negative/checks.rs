use super::gateway::Gateway;
use super::http::post_raw;

pub fn denials(gw: &Gateway, cap_a: &str) {
    assert!(matches_status(
        gw,
        "long.a.vau.place",
        Some("https://long.a.vau.place"),
        None,
        &[401, 403],
    ));
    assert_eq!(
        s(gw, "long.a.vau.place", "https://long.a.vau.place", "wrong"),
        401
    );
    assert_eq!(
        s(gw, "evil.example", "https://long.a.vau.place", cap_a),
        403
    );
    assert_eq!(
        s(gw, "long.a.vau.place", "https://evil.example", cap_a),
        403
    );
    assert_eq!(
        s(gw, "kawun.a.vau.place", "https://long.a.vau.place", cap_a),
        403
    );
}

pub fn cross_app(gw: &Gateway, cap_a: &str) {
    assert_eq!(
        s(gw, "kawun.a.vau.place", "https://kawun.a.vau.place", cap_a),
        401
    );
}

fn s(gw: &Gateway, host: &str, origin: &str, cap: &str) -> u16 {
    status(gw, host, Some(origin), Some(cap))
}

fn status(gw: &Gateway, host: &str, origin: Option<&str>, cap: Option<&str>) -> u16 {
    post_raw(gw.addr, host, origin, cap, b"x").0
}

fn matches_status(
    gw: &Gateway,
    host: &str,
    origin: Option<&str>,
    cap: Option<&str>,
    ok: &[u16],
) -> bool {
    ok.contains(&status(gw, host, origin, cap))
}
