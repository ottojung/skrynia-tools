use super::{Request, Unavailable, handle};
use std::collections::BTreeMap;

fn req(method: &str, path: &str) -> Request {
    Request {
        method: method.into(),
        path: path.into(),
        query: None,
        headers: BTreeMap::new(),
        body: Vec::new(),
    }
}

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
    assert!(allow.contains("If-Range"));
}

#[test]
fn unknown_path_is_not_found_with_cors() {
    let r = handle(&Unavailable, &req("GET", "/nope"));
    assert_eq!(r.status, 404);
    assert_eq!(r.body, br#"{"error":"not_found"}"#);
    assert_eq!(
        r.headers.get("access-control-expose-headers"),
        Some(&"ETag".into())
    );
}

#[test]
fn known_paths_reject_wrong_methods() {
    let r = handle(&Unavailable, &req("POST", "/health"));
    assert_eq!(r.status, 405);
    assert_eq!(r.body, br#"{"error":"method_not_allowed"}"#);
}
