use super::super::{Callback, InvokeError};
pub const MISSING: i32 = -2;
pub const MAX_DATA: usize = 64 * 1024;

pub fn call<I: Callback>(invoke: &I, body: &[u8]) -> Result<(i32, Vec<u8>), InvokeError> {
    let response = invoke.invoke(body)?;
    let (num, data) = response.split_at_checked(4).ok_or(InvokeError::Unavailable)?;
    let status = i32::from_le_bytes(num.try_into().map_err(|_| InvokeError::Unavailable)?);
    Ok((status, data.to_vec()))
}
pub fn ok(status: i32) -> Result<i32, InvokeError> {
    if status < 0 { Err(InvokeError::Unavailable) } else { Ok(status) }
}
