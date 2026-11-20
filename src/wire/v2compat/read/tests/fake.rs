use super::{
    CompatBackend, CompatResult, Meta, Object, ObjectRead, Request, Response, Store, Unavailable,
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
impl crate::wire::v2compat::Admin for Stub {}
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
