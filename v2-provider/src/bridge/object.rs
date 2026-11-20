use super::{Callback, InvokeBackend, mutate, wire};
use skrynia_tools::wire::v2compat::{
    CompatError, CompatResult, Mutation, Object, ObjectRead, Store,
};
impl<I: Callback> Store for InvokeBackend<'_, I> {
    fn read_object(&self, key: &str) -> CompatResult<ObjectRead> {
        let req = wire::key_req(1, key).ok_or_else(unavailable)?;
        let result = self.invoke.invoke(&req).map_err(|_| unavailable())?;
        match result.split_first() {
            Some((&0, [])) => Ok(ObjectRead::Missing),
            Some((&1, b)) if b.len() <= 8 * 1024 * 1024 => Ok(ObjectRead::Found(Object {
                key: key.into(),
                bytes: b.to_vec(),
            })),
            _ => Err(unavailable()),
        }
    }
    fn mutate_object(&self, req: Mutation<'_>) -> CompatResult<bool> {
        mutate::apply(self, req)
    }
}
pub(super) fn unavailable() -> CompatError {
    CompatError::new(503, "unavailable")
}
