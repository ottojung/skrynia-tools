use super::backend::{CompatBackend, CompatResult, Meta, Object, ObjectRead, Store};
use super::{Request, Response, Unavailable, handle};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
#[derive(Clone)]
struct R(Option<Vec<u8>>);
impl Meta for R {
    fn health(&self) -> Response {
        Unavailable.health()
    }
    fn version(&self) -> Response {
        Unavailable.health()
    }
    fn push_vapid(&self) -> Response {
        Unavailable.health()
    }
}
impl Store for R {
    fn read_object(&self, key: &str) -> CompatResult<ObjectRead> {
        Ok(match &self.0 {
            Some(b) => ObjectRead::Found(Object {
                key: key.into(),
                bytes: b.clone(),
            }),
            None => ObjectRead::Missing,
        })
    }
}
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
fn store_get_head_and_missing() {
    let b = b"hello".to_vec();
    let tag = format!("\"{}\"", hex::encode(Sha256::digest(&b)));
    let r = handle(&R(Some(b.clone())), &req("GET", "/store/a.txt"));
    assert_eq!(r.status, 200);
    assert_eq!(r.headers.get("etag"), Some(&tag));
    assert_eq!(r.body, b"hello");
    let h = handle(&R(Some(b)), &req("HEAD", "/store/a.txt"));
    assert!(h.status == 200 && h.body.is_empty());
    assert_eq!(handle(&R(None), &req("GET", "/store/a.txt")).status, 404);
}
