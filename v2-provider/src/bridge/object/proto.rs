use super::{Callback, CompatError};
pub const MISSING: i32 = -2;
pub const MAX_OBJECT: usize = 8 * 1024 * 1024;
fn unavailable() -> CompatError {
    CompatError::new(503, "unavailable")
}

pub fn call<I: Callback>(invoke: &I, body: &[u8]) -> Result<(i32, Vec<u8>), CompatError> {
    let response = invoke.invoke(body).map_err(|_| unavailable())?;
    let (number, data) = response.split_at_checked(4).ok_or_else(unavailable)?;
    let status = i32::from_le_bytes(number.try_into().map_err(|_| unavailable())?);
    if status < 0 && !data.is_empty() {
        return Err(unavailable());
    }
    if status >= 0
        && data.len()
            != match body.first() {
                Some(2) => status as usize,
                _ => 0,
            }
    {
        return Err(unavailable());
    }
    Ok((status, data.to_vec()))
}
pub fn close<I: Callback>(invoke: &I, fd: i32) -> Result<(), CompatError> {
    let mut req = vec![4];
    req.extend_from_slice(&fd.to_le_bytes());
    let (status, _) = call(invoke, &req)?;
    if status < 0 {
        Err(unavailable())
    } else {
        Ok(())
    }
}
pub fn denied() -> CompatError {
    unavailable()
}
