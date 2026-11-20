use super::super::{Callback, InvokeError};
use super::proto;
pub fn acquire<I: Callback>(i: &I) -> Result<i32, InvokeError> {
    let (status, data) = proto::call(i, &[14])?;
    if !data.is_empty() {
        return Err(InvokeError::Unavailable);
    }
    Ok(status)
}
pub fn release<I: Callback>(i: &I) -> Result<(), InvokeError> {
    let (status, data) = proto::call(i, &[15])?;
    if status != 0 || !data.is_empty() {
        return Err(InvokeError::Unavailable);
    }
    Ok(())
}
