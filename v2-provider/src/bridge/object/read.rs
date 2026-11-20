use super::proto;
use super::{Callback, CompatError};
pub fn run<I: Callback>(invoke: &I, fd: i32) -> Result<Vec<u8>, CompatError> {
    let mut out = Vec::new();
    loop {
        let mut req = vec![2u8];
        req.extend_from_slice(&fd.to_le_bytes());
        req.extend_from_slice(&65536u32.to_le_bytes());
        let (status, bytes) = proto::call(invoke, &req)?;
        if status < 0 {
            return Err(proto::denied());
        }
        if status == 0 {
            return Ok(out);
        }
        if bytes.len() != status as usize || out.len() + bytes.len() > proto::MAX_OBJECT {
            return Err(proto::denied());
        }
        out.extend_from_slice(&bytes);
    }
}
