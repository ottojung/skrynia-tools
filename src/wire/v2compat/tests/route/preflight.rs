use super::*;
#[test]
fn preflight_has_cors_and_allow_headers() {
    let r = handle(&Unavailable, &req("OPTIONS", "/store/x"));
    assert_eq!(r.status, 204);
    assert_eq!(
        r.headers.get("access-control-allow-origin"),
        Some(&"*".into())
    );
    let allow = r.headers.get("access-control-allow-headers").unwrap();
    assert!(allow.contains("X-Skrynia-If-SHA256"));
}
