#[allow(unused_imports)]
pub(super) use super::Request as Req;
use super::api_backend::{
    CompatBackend, CompatResult, Meta, Mutation, MutationKind, Object, ObjectRead, Store,
};
pub(super) use super::api_backend::{Meta as PMeta, Push};
use super::{Response, Unavailable};
pub(super) use super::{Response as Resp, Unavailable as Un};
use sha2::Digest;
use std::sync::Mutex;
mod push;

#[derive(Default)]
pub struct Memory(pub Mutex<Vec<MutationKind>>);
impl Meta for Memory {
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
impl Store for Memory {
    fn read_object(&self, key: &str) -> CompatResult<ObjectRead> {
        Ok(ObjectRead::Found(Object {
            key: key.into(),
            bytes: b"old".to_vec(),
        }))
    }
    fn mutate_object(&self, req: Mutation<'_>) -> CompatResult<bool> {
        assert_eq!(req.key, "a.txt");
        assert_eq!(req.token.len(), 64);
        if let Some(hash) = req.expected {
            let actual = hex::encode(sha2::Sha256::digest(b"old"));
            if hash != actual {
                return Err(super::api_backend::CompatError::new(
                    412,
                    "content_mismatch",
                ));
            }
        }
        self.0.lock().unwrap().push(req.kind);
        Ok(matches!(req.kind, MutationKind::Create))
    }
}
impl super::api_backend::GrantAdmin for Memory {}
impl CompatBackend for Memory {}
