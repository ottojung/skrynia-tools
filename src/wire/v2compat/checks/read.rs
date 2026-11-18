use super::{
    Request,
    backend::{CompatBackend, Meta, ObjectRead, Store},
    handle,
    types::{Object, Response},
};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
#[derive(Clone)]
struct R(Option<Vec<u8>>);
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
impl Store for R {
    fn read_object(&self, key: &str) -> super::backend::CompatResult<ObjectRead> {
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
fn req(method: &str, path: &str) -> Request {
    Request {
        method: method.into(),
        path: path.into(),
        query: None,
        headers: BTreeMap::new(),
        body: Vec::new(),
    }
}
fn etag(b: &[u8]) -> String {
    format!("\"{}\"", hex::encode(Sha256::digest(b)))
}
#[test]
fn store_get_missing_and_found_headers() {
    let n = handle(&R(None), &req("GET", "/store/a.txt"));
    assert_eq!(n.status, 404);
    assert_eq!(n.body, br#"{"error":"not_found"}"#);
    let b = b"hello";
    let r = handle(&R(Some(b.to_vec())), &req("GET", "/store/a.txt"));
    assert_eq!(r.status, 200);
    assert_eq!(r.headers.get("etag"), Some(&etag(b)));
    assert_eq!(
        r.headers.get("content-type"),
        Some(&"text/plain; charset=utf-8".into())
    );
    assert_eq!(r.headers.get("cache-control"), Some(&"no-cache".into()));
    assert_eq!(r.headers.get("accept-ranges"), Some(&"bytes".into()));
    assert_eq!(r.body, b);
}
