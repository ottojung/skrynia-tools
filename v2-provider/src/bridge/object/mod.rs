mod open;
mod proto;
mod read;
use super::{Callback, InvokeBackend};
use crate::durable;
use skrynia_tools::wire::v2compat::{CompatError, Mutation, Object, ObjectRead, Store};

impl<I: Callback> Store for InvokeBackend<'_, I> {
    fn read_object(&self, key: &str) -> Result<ObjectRead, CompatError> {
        let Some(fd) = open::run(self.invoke, key)? else {
            return Ok(ObjectRead::Missing);
        };
        let value = read::run(self.invoke, fd);
        let closed = proto::close(self.invoke, fd);
        let bytes = value?;
        closed?;
        Ok(ObjectRead::Found(Object { key: key.into(), bytes }))
    }
    fn mutate_object(&self, request: Mutation<'_>) -> Result<bool, CompatError> {
        durable::ops::mutate(self.invoke, request)
    }
}
