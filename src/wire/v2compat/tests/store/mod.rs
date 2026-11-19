use super::backend::{CompatBackend, CompatResult, Meta, Object, ObjectRead, Store};
pub(super) use super::backend::{Meta as PMeta, Push};
pub(super) use super::{Request as Req, Response as Resp, Unavailable as Un};
use super::{Request, Response, Unavailable, handle};
use std::collections::BTreeMap;
mod case;
mod push;
#[derive(Clone)]
pub(super) struct R(pub(super) Option<Vec<u8>>);
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
pub(super) fn req(m: &str, p: &str) -> Request {
    Request {
        method: m.into(),
        path: p.into(),
        query: None,
        headers: BTreeMap::new(),
        body: vec![],
    }
}
