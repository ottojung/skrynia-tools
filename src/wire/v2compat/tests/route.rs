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
}
#[test]
fn unknown_and_method_errors() {
    let n = handle(&Unavailable, &req("GET", "/nope"));
    assert_eq!(n.status, 404);
    let m = handle(&Unavailable, &req("POST", "/health"));
    assert_eq!(m.status, 405);
    let k = handle(&Unavailable, &req("GET", "/store/bad%2"));
    assert_eq!(k.status, 400);
    assert_eq!(k.body, br#"{"error":"invalid_percent_encoding"}"#);
}
#[test]
fn recognized_mutations_route_as_not_implemented() {
    let c = handle(&Unavailable, &req("POST", "/store/a.txt"));
    assert_eq!(c.status, 401);
    assert_eq!(c.body, br#"{"error":"grant_user_token_required"}"#);
    let g = handle(&Unavailable, &req("POST", "/grants"));
    assert_eq!(g.status, 501);
}
