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
fn route_stubs_auth_and_methods_stay_consistent() {
    let r = R(Some(b"abcdef".to_vec()));
    assert_eq!(handle(&Unavailable, &req("GET", "/store")).status, 501);
    assert_eq!(handle(&Unavailable, &req("POST", "/grants")).status, 401);
    assert_eq!(handle(&r, &req("GET", "/grants/x")).status, 401);
    assert_eq!(handle(&r, &req("POST", "/push/subscriptions")).status, 501);
    assert_eq!(handle(&r, &req("TRACE", "/store/a.txt")).status, 405);
    assert_eq!(handle(&r, &req("GET", "/missing")).status, 404);
}
