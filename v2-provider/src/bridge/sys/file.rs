use super::super::{Callback, InvokeError};
use super::proto;
pub fn open_disk<I: Callback>(i: &I, path: &str, mode: u32) -> Result<i32, InvokeError> {
    let mut r = vec![1u8];
    r.extend_from_slice(&(path.len() as u32).to_le_bytes());
    r.extend_from_slice(path.as_bytes());
    r.extend_from_slice(&mode.to_le_bytes());
    proto::ok(proto::call(i, &r)?.0)
}
pub fn read<I: Callback>(i: &I, fd: i32, len: u32) -> Result<(i32, Vec<u8>), InvokeError> {
    let mut r = vec![2u8];
    r.extend_from_slice(&fd.to_le_bytes());
    r.extend_from_slice(&len.to_le_bytes());
    proto::call(i, &r)
}
pub fn write<I: Callback>(i: &I, fd: i32, bytes: &[u8]) -> Result<i32, InvokeError> {
    let mut r = vec![3u8];
    r.extend_from_slice(&fd.to_le_bytes());
    r.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
    r.extend_from_slice(bytes);
    proto::ok(proto::call(i, &r)?.0)
}
pub fn close<I: Callback>(i: &I, fd: i32) -> Result<i32, InvokeError> {
    let mut r = vec![4u8];
    r.extend_from_slice(&fd.to_le_bytes());
    Ok(proto::call(i, &r)?.0)
}
