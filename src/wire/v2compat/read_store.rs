use super::{
    backend::{CompatBackend, ObjectRead},
    key, mime, reply,
    types::{Request, Response},
};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

pub fn handle<B: CompatBackend>(b: &B, method: &str, raw: &str, req: &Request) -> Response {
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
    if req.headers.get("if-none-match").map(|v| v.trim()) == Some(tag.as_str()) {
        let mut h = BTreeMap::new();
        h.insert("etag".into(), tag);
        return Response {
            status: 304,
            headers: h,
            body: Vec::new(),
        };
    }
    let mut h = BTreeMap::new();
    h.insert("content-type".into(), mime::for_key(&obj.key).into());
    h.insert("etag".into(), tag);
    h.insert("cache-control".into(), "no-cache".into());
    h.insert("accept-ranges".into(), "bytes".into());
    h.insert("x-content-type-options".into(), "nosniff".into());
    h.insert("content-length".into(), obj.bytes.len().to_string());
    Response {
        status: 200,
        headers: h,
        body: if method == "HEAD" {
            Vec::new()
        } else {
            obj.bytes
        },
    }
}
