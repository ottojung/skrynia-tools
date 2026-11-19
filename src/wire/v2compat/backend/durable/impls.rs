use super::core::DurableBackend;
use super::data::{RandomSource, StateStore};
use super::reply;
use super::{CompatBackend, CompatResult, ListPage, Meta, Object, ObjectRead, Response, Store};

impl<S, R> Meta for DurableBackend<S, R> {
    fn health(&self) -> Response {
        reply::json(200, b"{\"status\":\"ok\"}".to_vec())
    }
    fn version(&self) -> Response {
        reply::json(200, b"{\"version\":\"v3-facade\"}".to_vec())
    }
    fn push_vapid(&self) -> Response {
        reply::json(501, b"{\"error\":\"not_implemented\"}".to_vec())
    }
}
impl<S: StateStore, R: RandomSource> Store for DurableBackend<S, R> {
    fn read_object(&self, key: &str) -> CompatResult<ObjectRead> {
        let state = self.checked()?;
        Ok(match state.objects.get(key) {
            Some(bytes) => ObjectRead::Found(Object {
                key: key.into(),
                bytes: bytes.clone(),
            }),
            None => ObjectRead::Missing,
        })
    }
    fn mutate_object(&self, req: super::Mutation<'_>) -> CompatResult<bool> {
        super::mutate::mutate(self, req)
    }
    fn list_objects(
        &self,
        token: &str,
        locations: &str,
        after: Option<&str>,
        limit: usize,
    ) -> CompatResult<ListPage> {
        super::core::list_objects(self, token, locations, after, limit)
    }
}
impl<S: StateStore, R: RandomSource> CompatBackend for DurableBackend<S, R> {}
