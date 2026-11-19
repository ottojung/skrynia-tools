use super::core::DurableBackend;
use super::data::{RandomSource, StateStore};
use super::{CompatBackend, CompatResult, Meta, Object, ObjectRead, Response, Store};
use std::collections::BTreeMap;

fn stub501() -> Response {
    let body = b"{\"error\":\"not_implemented\"}".to_vec();
    let mut headers = BTreeMap::new();
    headers.insert("content-length".into(), body.len().to_string());
    headers.insert("content-type".into(), "application/json".into());
    headers.insert("cache-control".into(), "no-store".into());
    headers.insert("access-control-allow-origin".into(), "*".into());
    headers.insert("access-control-expose-headers".into(), "ETag".into());
    Response {
        status: 501,
        headers,
        body,
    }
}
fn read_obj<S, R>(backend: &DurableBackend<S, R>, key: &str) -> CompatResult<ObjectRead> {
    let state = backend.lock();
    Ok(match state.objects.get(key) {
        Some(bytes) => ObjectRead::Found(Object {
            key: key.into(),
            bytes: bytes.clone(),
        }),
        None => ObjectRead::Missing,
    })
}
impl<S, R> Meta for DurableBackend<S, R> {
    fn health(&self) -> Response {
        stub501()
    }
    fn version(&self) -> Response {
        stub501()
    }
    fn push_vapid(&self) -> Response {
        stub501()
    }
}
impl<S: StateStore, R: RandomSource> Store for DurableBackend<S, R> {
    fn read_object(&self, key: &str) -> CompatResult<ObjectRead> {
        read_obj(self, key)
    }
    fn mutate_object(&self, req: super::Mutation<'_>) -> CompatResult<bool> {
        super::mutate::mutate(self, req)
    }
}
impl<S: StateStore, R: RandomSource> CompatBackend for DurableBackend<S, R> {}
