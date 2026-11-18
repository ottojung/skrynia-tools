use super::{
    Request,
    backend::{CompatBackend, Meta, Store},
    handle,
    types::Response,
};
use std::collections::BTreeMap;
#[derive(Clone)]
struct R;
impl Meta for R {
    fn health(&self) -> Response {
        Response {
            status: 200,
            headers: BTreeMap::new(),
            body: br#"{"ok":true}"#.to_vec(),
        }
    }
    fn version(&self) -> Response {
        self.health()
    }
    fn push_vapid(&self) -> Response {
        self.health()
    }
}
impl Store for R {}
impl CompatBackend for R {}
fn req(m: &str, p: &str) -> Request {
    Request {
        method: m.into(),
        path: p.into(),
        query: None,
        headers: BTreeMap::new(),
        body: vec![],
    }
}
#[test]
fn store_key_validation_and_methods() {
    let b = &R;
    for (p, code) in [
        ("/store/bad%", "invalid_percent_encoding"),
        ("/store/.hidden", "invalid_key"),
        ("/store/a/../b", "invalid_key"),
    ] {
        let r = handle(b, &req("GET", p));
        assert_eq!(r.status, 400, "{p}");
        assert_eq!(
            r.body,
            format!("{{\"error\":\"{code}\"}}").into_bytes(),
            "{p}"
        );
    }
    let m = handle(b, &req("POST", "/store/x"));
    assert_eq!(m.status, 405);
    assert_eq!(m.body, br#"{"error":"method_not_allowed"}"#);
}
