use super::{MAX_FRAME, digest};
use anyhow::{Result, bail};

fn body(frame: &[u8]) -> Result<(u8, &[u8])> {
    if frame.len() < 6 || frame.len() > MAX_FRAME || frame[0] != 1 || frame[1] > 1 {
        bail!("invalid response frame");
    }
    let len = u32::from_le_bytes(frame[2..6].try_into()?) as usize;
    if len != frame.len() - 6 {
        bail!("invalid response length");
    }
    Ok((frame[1], &frame[6..]))
}
fn conflict(data: &[u8]) -> bool {
    data == [10, 0] || (data.len() == 34 && data.starts_with(&[10, 1]))
}
pub fn read(frame: &[u8]) -> Result<Option<Vec<u8>>> {
    match body(frame)? {
        (0, data) => Ok(Some(data.to_vec())),
        (1, [8]) => Ok(None),
        _ => bail!("application read rejected"),
    }
}
pub fn written(frame: &[u8], value: &[u8]) -> Result<bool> {
    match body(frame)? {
        (0, data) if data == digest(value) => Ok(true),
        (1, data) if conflict(data) => Ok(false),
        _ => bail!("write rejected or outcome unknown"),
    }
}
pub fn deleted(frame: &[u8]) -> Result<bool> {
    match body(frame)? {
        (0, []) => Ok(true),
        (1, data) if conflict(data) => Ok(false),
        _ => bail!("delete rejected or outcome unknown"),
    }
}
