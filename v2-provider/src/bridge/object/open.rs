use super::proto;
use super::{Callback, CompatError};
pub fn run<I: Callback>(invoke: &I, key: &str) -> Result<Option<i32>, CompatError> {
    // This is an application-owned namespace, not a kernel v2 path.
    let name = format!("v2/objects/{key}");
    let mut req = vec![1u8];
    req.extend_from_slice(&(name.len() as u32).to_le_bytes());
    req.extend_from_slice(name.as_bytes());
    req.extend_from_slice(&1u32.to_le_bytes()); // read-only
    let (status, _) = proto::call(invoke, &req)?;
    match status {
        proto::MISSING => Ok(None),
        n if n < 0 => Err(proto::denied()),
        n => Ok(Some(n)),
    }
}
