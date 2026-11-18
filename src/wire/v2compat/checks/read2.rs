use super::{
    Request,
    backend::{CompatBackend, Meta, ObjectRead, Store},
    handle,
    types::{Object, Response},
};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
#[derive(Clone)]
struct R(Vec<u8>);
impl Meta for R {
    fn health(&self) -> Response {
        Response {
            status: 200,
            headers: BTreeMap::new(),
            body: vec![],
        }
    }
    fn version(&self) -> Response {
        self.health()
    }
    fn push_vapid(&self) -> Response {
        self.health()
    }
}
impl Store for R {
    fn read_object(&self, k: &str) -> super::backend::CompatResult<ObjectRead> {
        Ok(ObjectRead::Found(Object {
            key: k.into(),
            bytes: self.0.clone(),
        }))
    }
}
impl CompatBackend for R {}
fn req(m: &str, p: &str, h: Vec<(&str, &str)>) -> Request {
    let mut headers = BTreeMap::new();
    for (k, v) in h {
        headers.insert(k.into(), v.into());
    }
    Request {
        method: m.into(),
        path: p.into(),
        query: None,
        headers,
        body: vec![],
    }
}
fn etag(b: &[u8]) -> String {
    format!("\"{}\"", hex::encode(Sha256::digest(b)))
}
#[test]
fn head_conditional_and_range() {
    let b = b"abcdef".to_vec();
    let t = etag(&b);
    let be = &R(b.clone());
    let h = handle(be, &req("HEAD", "/store/x", vec![]));
    assert_eq!(h.status, 200);
    assert_eq!(h.body.len(), 0);
    assert_eq!(h.headers.get("content-length"), Some(&"6".into()));
    let n = handle(
        be,
        &req("GET", "/store/x", vec![("if-none-match", t.as_str())]),
    );
    assert_eq!(n.status, 304);
    assert_eq!(n.body.len(), 0);
    let p = handle(be, &req("GET", "/store/x", vec![("range", "bytes=1-3")]));
    assert_eq!(p.status, 206);
    assert_eq!(p.body, b"bcd");
    assert_eq!(p.headers.get("content-range"), Some(&"bytes 1-3/6".into()));
    let u = handle(be, &req("GET", "/store/x", vec![("range", "bytes=9-10")]));
    assert_eq!(u.status, 416);
    assert_eq!(u.headers.get("content-range"), Some(&"bytes */6".into()));
}
