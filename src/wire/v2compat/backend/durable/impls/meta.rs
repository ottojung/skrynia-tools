use super::{DurableBackend, Meta, Response};
use std::collections::BTreeMap;

fn stub501() -> Response {
    let body = b"{\"error\":\"not_implemented\"}".to_vec();
    let mut h = BTreeMap::new();
    h.insert("content-length".into(), body.len().to_string());
    h.insert("content-type".into(), "application/json".into());
    h.insert("cache-control".into(), "no-store".into());
    h.insert("access-control-allow-origin".into(), "*".into());
    h.insert("access-control-expose-headers".into(), "ETag".into());
    Response {
        status: 501,
        headers: h,
        body,
    }
}
impl<S, R> Meta for DurableBackend<S, R> {
    fn health(&self) -> Response {
        super::reply::json(200, b"{\"status\":\"ok\"}".to_vec())
    }
    fn version(&self) -> Response {
        super::reply::json(200, b"{\"version\":\"v3-facade\"}".to_vec())
    }
    fn push_vapid(&self) -> Response {
        stub501()
    }
}
