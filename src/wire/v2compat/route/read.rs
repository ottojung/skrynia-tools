use super::{
    super::{
        backend::{CompatBackend, ObjectRead},
        reply,
        types::{Request, Response},
    },
    headers, key,
};
use sha2::{Digest, Sha256};

pub fn store<B: CompatBackend>(b: &B, method: &str, raw: &str, req: &Request) -> Response {
    let key = match key::decode(raw) {
        Ok(v) => v,
        Err(e) => return reply::error(400, e),
    };
    let obj = match b.read_object(&key) {
        Ok(ObjectRead::Found(o)) => o,
        Ok(ObjectRead::Missing) => return reply::error(404, "not_found"),
        Err(e) => return reply::error(e.status, &e.code),
    };
    let tag = format!("\"{}\"", hex::encode(Sha256::digest(&obj.bytes)));
    headers::respond(method, &obj.key, &obj.bytes, &tag, req)
}
