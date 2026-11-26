//! Length-prefixed callback frames for P1-mediated nested guest calls.

use super::ops;
use super::stream::{self, Transport};

pub fn u32_at(bytes: &[u8], at: usize) -> Result<usize, i64> {
    let raw: [u8; 4] = bytes
        .get(at..at + 4)
        .ok_or(ops::ERR_INVAL)?
        .try_into()
        .map_err(|_| ops::ERR_INVAL)?;
    Ok(u32::from_le_bytes(raw) as usize)
}

pub fn frame(req: &[u8]) -> Vec<u8> {
    let mut out = (req.len() as u32).to_le_bytes().to_vec();
    out.extend_from_slice(req);
    out
}

pub fn deframe(bytes: &[u8]) -> Result<Vec<Vec<u8>>, i64> {
    let mut out = Vec::new();
    let mut at = 0;
    while at < bytes.len() {
        let len = u32_at(bytes, at)?;
        at += 4;
        let end = at.checked_add(len).ok_or(ops::ERR_INVAL)?;
        out.push(bytes.get(at..end).ok_or(ops::ERR_INVAL)?.to_vec());
        at = end;
    }
    Ok(out)
}

pub fn cb_drain<T: Transport>(t: &T, ctx: i32) -> Result<Vec<Vec<u8>>, i64> {
    let mut raw = Vec::new();
    loop {
        let byte = t.invoke(ctx, ops::CB_GET, 0);
        if byte == ops::EOF {
            break;
        }
        if byte < 0 {
            return Err(byte);
        }
        raw.push(byte as u8);
    }
    deframe(&raw)
}

pub fn cb_reply<T: Transport>(t: &T, ctx: i32, res: &[u8]) -> Result<(), i64> {
    stream::send_cb(t, ctx, &frame(res))
}
