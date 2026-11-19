use super::{
    CompatBackend, CompatResult, DurableBackend, ListPage, Mutation, Object, ObjectRead,
    RandomSource, StateStore, Store, mutate,
};

fn read_obj<S, R>(backend: &DurableBackend<S, R>, key: &str) -> CompatResult<ObjectRead> {
    let state = backend.checked()?;
    Ok(match state.objects.get(key) {
        Some(bytes) => ObjectRead::Found(Object {
            key: key.into(),
            bytes: bytes.clone(),
        }),
        None => ObjectRead::Missing,
    })
}
impl<S: StateStore, R: RandomSource> Store for DurableBackend<S, R> {
    fn read_object(&self, key: &str) -> CompatResult<ObjectRead> {
        read_obj(self, key)
    }
    fn mutate_object(&self, req: Mutation<'_>) -> CompatResult<bool> {
        mutate::mutate(self, req)
    }
    fn list_objects(
        &self,
        token: &str,
        locations: &str,
        after: Option<&str>,
        limit: usize,
    ) -> CompatResult<ListPage> {
        super::super::core::list_objects(self, token, locations, after, limit)
    }
}
impl<S: StateStore, R: RandomSource> CompatBackend for DurableBackend<S, R> {}
