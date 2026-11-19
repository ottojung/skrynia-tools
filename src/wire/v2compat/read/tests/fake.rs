use super::{
    CompatBackend, CompatResult, GrantAdmin, Meta, Object, ObjectRead, Push, Request, Response,
    Store, Unavailable,
};
use std::collections::BTreeMap;
pub struct Stub(pub Vec<u8>);
impl Store for Stub {
    fn read_object(&self, key: &str) -> CompatResult<ObjectRead> {
        Ok(ObjectRead::Found(Object {
            key: key.into(),
            bytes: self.0.clone(),
        }))
    }
}
impl Meta for Stub {
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

impl Push for Stub {
    fn push_subscribe(&self, _req: &Request) -> Response {
        Unavailable.health()
    }
    fn push_update(&self, _id: &str, _req: &Request) -> Response {
        Unavailable.health()
    }
    fn push_delete(&self, _id: &str, _req: &Request) -> Response {
        Unavailable.health()
    }
}

impl GrantAdmin for Stub {}
impl CompatBackend for Stub {}
pub fn fake() -> Stub {
    Stub(b"abcdef".to_vec())
}
pub fn request(method: &str) -> Request {
    Request {
        method: method.into(),
        path: "/store/a.txt".into(),
        query: None,
        headers: BTreeMap::new(),
        body: Vec::new(),
    }
}
