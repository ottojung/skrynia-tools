pub(super) use super::{GrantAdmin, Push};
use super::{
    backend::{CompatBackend, ObjectRead},
    key, mime, reply,
    types::{Request, Response},
};
use sha2::{Digest, Sha256};
pub(super) mod cache;
mod range;
mod response;
mod select;
#[cfg(test)]
mod tests;

pub fn handle<B: CompatBackend>(backend: &B, method: &str, raw: &str, req: &Request) -> Response {
    let key = match key::decode(raw) {
        Ok(v) => v,
        Err(e) => return reply::error(400, e),
    };
    let obj = match backend.read_object(&key) {
        Ok(ObjectRead::Found(o)) => o,
        Ok(ObjectRead::Missing) => return reply::error(404, "not_found"),
        Err(e) => return reply::error(e.status, &e.code),
    };
    let tag = format!("\"{}\"", hex::encode(Sha256::digest(&obj.bytes)));
    let selected = select::choose(&req.headers, &tag, obj.bytes.len());
    response::build(method, &obj.key, &tag, obj.bytes, selected)
}

#[cfg(test)]
pub(super) use super::backend::{CompatResult, Meta, Object, Store, Unavailable};
