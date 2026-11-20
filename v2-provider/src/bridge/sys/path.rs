use super::super::{Callback, InvokeError};
use super::proto;
fn one<I: Callback>(i: &I, op: u8, p: &str) -> Result<i32, InvokeError> {
    let mut r = vec![op];
    r.extend_from_slice(&(p.len() as u32).to_le_bytes());
    r.extend_from_slice(p.as_bytes());
    Ok(proto::call(i, &r)?.0)
}
pub fn sync<I: Callback>(i: &I, fd: i32) -> Result<i32, InvokeError> {
    let mut r = vec![5u8];
    r.extend_from_slice(&fd.to_le_bytes());
    Ok(proto::call(i, &r)?.0)
}
pub fn rename<I: Callback>(i: &I, a: &str, b: &str) -> Result<i32, InvokeError> {
    let mut r = vec![8u8];
    r.extend_from_slice(&(a.len() as u32).to_le_bytes());
    r.extend_from_slice(a.as_bytes());
    r.extend_from_slice(&(b.len() as u32).to_le_bytes());
    r.extend_from_slice(b.as_bytes());
    Ok(proto::call(i, &r)?.0)
}
pub fn delete<I: Callback>(i: &I, p: &str) -> Result<i32, InvokeError> {
    one(i, 9, p)
}
pub fn mkdir<I: Callback>(i: &I, p: &str, m: u32) -> Result<i32, InvokeError> {
    let mut r = vec![10u8];
    r.extend_from_slice(&(p.len() as u32).to_le_bytes());
    r.extend_from_slice(p.as_bytes());
    r.extend_from_slice(&m.to_le_bytes());
    Ok(proto::call(i, &r)?.0)
}
pub fn exists<I: Callback>(i: &I, p: &str) -> Result<i32, InvokeError> {
    one(i, 11, p)
}
pub fn is_dir<I: Callback>(i: &I, p: &str) -> Result<i32, InvokeError> {
    one(i, 12, p)
}
