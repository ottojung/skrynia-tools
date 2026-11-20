use super::super::{Callback, InvokeError};
use super::proto;
pub fn open<I: Callback>(i: &I) -> Result<i32, InvokeError> {
    proto::ok(proto::call(i, &[16u8])?.0)
}
pub fn cas_u64<I: Callback>(
    i: &I,
    fd: i32,
    off: u64,
    exp: u64,
    des: u64,
) -> Result<(i32, u64), InvokeError> {
    let mut r = vec![17u8];
    r.extend_from_slice(&fd.to_le_bytes());
    r.extend_from_slice(&off.to_le_bytes());
    r.extend_from_slice(&exp.to_le_bytes());
    r.extend_from_slice(&des.to_le_bytes());
    let (st, data) = proto::call(i, &r)?;
    if st < 0 {
        return Ok((st, 0));
    }
    let old = data.get(..8).ok_or(InvokeError::Unavailable)?;
    Ok((st, u64::from_le_bytes(old.try_into().unwrap())))
}
