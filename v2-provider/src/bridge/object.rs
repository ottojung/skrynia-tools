use super::{Callback, InvokeBackend};
use skrynia_tools::wire::v2compat::{CompatError, Object, ObjectRead, Store};
impl<I: Callback> Store for InvokeBackend<'_, I> {
    fn read_object(&self, key: &str) -> Result<ObjectRead, CompatError> {
        let mut request = vec![1u8]; // app-defined v3 read-object callback
        let len = u16::try_from(key.len()).map_err(|_| unavailable())?;
        request.extend_from_slice(&len.to_le_bytes());
        request.extend_from_slice(key.as_bytes());
        let result = self.invoke.invoke(&request).map_err(|_| unavailable())?;
        match result.split_first() {
            Some((&0, [])) => Ok(ObjectRead::Missing),
            Some((&1, bytes)) if bytes.len() <= 8 * 1024 * 1024 => Ok(ObjectRead::Found(Object {
                key: key.into(),
                bytes: bytes.to_vec(),
            })),
            _ => Err(unavailable()),
        }
    }
}
fn unavailable() -> CompatError {
    CompatError::new(503, "unavailable")
}
