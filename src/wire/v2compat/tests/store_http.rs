use super::backend::{Admin, CompatBackend, CompatResult, Meta, Object, ObjectRead, Store};
use super::{Request, Response, Unavailable, handle};
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
fn preflight_works() {
    let mut opts = Request {
        method: "OPTIONS".into(),
        path: "/store".into(),
        query: None,
        headers: BTreeMap::new(),
        body: vec![],
    };
    opts.headers.insert("origin".into(), "x".into());
    let r = handle(&R(Some(b"abcdef".to_vec())), &opts);
    assert_eq!(r.status, 204);
    assert_eq!(r.headers["access-control-allow-origin"], "*");
}
