use super::super::{Callback, InvokeError};
use super::proto;
pub fn bytes<I: Callback>(i: &I, n: usize) -> Result<Vec<u8>, InvokeError> {
    let bits = (n as u32).saturating_mul(8);
    let mut r = vec![15u8];
    r.extend_from_slice(&bits.to_le_bytes());
    let (st, data) = proto::call(i, &r)?;
    if st < 0 || data.len() < n {
        return Err(InvokeError::Unavailable);
    }
    Ok(data[..n].to_vec())
}
