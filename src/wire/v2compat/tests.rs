use super::{
    Request, Response,
    backend::{CompatBackend, Meta, Store},
    handle,
};
use std::collections::BTreeMap;
#[derive(Clone)]
struct Fixed;
impl Meta for Fixed {
    fn health(&self) -> Response {
        j(200, br#"{"ok":true}"#)
    }
    fn version(&self) -> Response {
        j(200, br#"{"version":"v3-test"}"#)
    }
    fn push_vapid(&self) -> Response {
        j(200, br#"{"publicKey":"k"}"#)
    }
}
impl Store for Fixed {}
impl CompatBackend for Fixed {}
fn j(status: u16, body: &[u8]) -> Response {
    let mut h = BTreeMap::new();
    h.insert("content-type".into(), "application/json".into());
    Response {
        status,
        headers: h,
        body: body.to_vec(),
    }
}
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
fn routes_public_meta_with_cors() {
    let r = handle(&Fixed, &req("GET", "/health"));
    assert_eq!(r.status, 200);
    assert_eq!(
        r.headers.get("access-control-allow-origin"),
        Some(&"*".into())
    );
    assert_eq!(r.body, br#"{"ok":true}"#);
}
