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
fn preflight_and_not_found() {
    let p = handle(&Fixed, &req("OPTIONS", "/store/x"));
    assert_eq!(p.status, 204);
    assert!(
        p.headers
            .get("access-control-allow-headers")
            .unwrap()
            .contains("X-Skrynia-If-SHA256")
    );
    let n = handle(&Fixed, &req("GET", "/nope"));
    assert_eq!(n.status, 404);
}
