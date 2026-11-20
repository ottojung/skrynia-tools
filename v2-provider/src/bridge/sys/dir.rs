use super::super::{Callback, InvokeError};
use super::proto;
pub fn open<I: Callback>(i: &I, p: &str) -> Result<i32, InvokeError> {
    let mut r = vec![9u8];
    r.extend_from_slice(&(p.len() as u32).to_le_bytes());
    r.extend_from_slice(p.as_bytes());
    proto::ok(proto::call(i, &r)?.0)
}
pub fn next<I: Callback>(i: &I, fd: i32) -> Result<(i32, Vec<u8>), InvokeError> {
    let mut r = vec![10u8];
    r.extend_from_slice(&fd.to_le_bytes());
    r.extend_from_slice(&4096u32.to_le_bytes());
    proto::call(i, &r)
}
