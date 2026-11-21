mod put;
pub use put::{app_put_absent, app_put_present};

use alloc::vec::Vec;
pub const VERSION: u8 = 1;
pub const OP_APP_GET: u8 = 7;
pub const OP_APP_PUT: u8 = 6;
pub const STATUS_OK: u8 = 0;
pub const STATUS_ERR: u8 = 1;
pub const PRECOND_ABSENT: u8 = 0;
pub const PRECOND_PRESENT: u8 = 1;

pub fn put_id(out: &mut Vec<u8>, id: &[u8]) {
    out.extend_from_slice(&(id.len() as u16).to_le_bytes());
    out.extend_from_slice(id);
}
pub fn put_blob(out: &mut Vec<u8>, b: &[u8]) {
    out.extend_from_slice(&(b.len() as u32).to_le_bytes());
    out.extend_from_slice(b);
}
pub fn app_get(cap: &[u8], key: &[u8]) -> Vec<u8> {
    let mut out = alloc::vec![VERSION, OP_APP_GET];
    put_id(&mut out, cap);
    put_blob(&mut out, key);
    out
}
pub fn decode_status(resp: &[u8]) -> Result<(u8, &[u8]), ()> {
    if resp.len() < 6 || resp[0] != VERSION {
        return Err(());
    }
    let blen = u32::from_le_bytes(resp[2..6].try_into().map_err(|_| ())?) as usize;
    let body = resp.get(6..6 + blen).ok_or(())?;
    Ok((resp[1], body))
}
