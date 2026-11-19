mod push;
use skrynia_tools::wire::v2compat::{
    CompatBackend, CompatResult, Meta, Object, ObjectRead, Response, Store, Unavailable,
};
#[derive(Default)]
pub struct MemoryStore;
impl Store for MemoryStore {
    fn read_object(&self, key: &str) -> CompatResult<ObjectRead> {
        if key != "a.txt" {
            return Ok(ObjectRead::Missing);
        }
        Ok(ObjectRead::Found(Object {
            key: key.into(),
            bytes: b"abcdef".to_vec(),
        }))
    }
}
impl Meta for MemoryStore {
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

impl skrynia_tools::wire::v2compat::GrantAdmin for MemoryStore {}
impl CompatBackend for MemoryStore {}
