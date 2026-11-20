use super::backend::{Admin, CompatBackend, CompatResult, Meta, Object, ObjectRead, Store};
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
impl Admin for R {}
impl CompatBackend for R {}
#[test]
fn conditional_head_works() {
    let mut head = Request {
        method: "HEAD".into(),
        path: "/store/a.txt".into(),
        query: None,
        headers: BTreeMap::new(),
        body: vec![],
    };
    head.headers.insert(
        "if-none-match".into(),
        format!("\"{}\"", hex::encode(Sha256::digest(b"abcdef"))),
    );
    assert_eq!(handle(&R(Some(b"abcdef".to_vec())), &head).status, 304);
}
