use super::{MAX_FRAME, valid_key};
use anyhow::{Result, bail};

fn prefix(op: u8, cap: &[u8; 32], key: &str) -> Result<Vec<u8>> {
    if !valid_key(key) {
        bail!("invalid application key");
    }
    let mut out = vec![1, op, 32, 0];
    out.extend_from_slice(cap);
    out.extend_from_slice(&(key.len() as u32).to_le_bytes());
    out.extend_from_slice(key.as_bytes());
    Ok(out)
}
pub fn get(cap: &[u8; 32], key: &str) -> Result<Vec<u8>> {
    prefix(7, cap, key)
}
pub fn put(
    cap: &[u8; 32],
    key: &str,
    expected: Option<&[u8; 32]>,
    value: &[u8],
) -> Result<Vec<u8>> {
    let mut out = prefix(6, cap, key)?;
    out.push(u8::from(expected.is_some()));
    if let Some(hash) = expected {
        out.extend_from_slice(hash);
    }
    if value.len() > MAX_FRAME - out.len() - 4 {
        bail!("request exceeds frame limit");
    }
    out.extend_from_slice(&(value.len() as u32).to_le_bytes());
    out.extend_from_slice(value);
    Ok(out)
}
pub fn delete(cap: &[u8; 32], key: &str, expected: &[u8; 32]) -> Result<Vec<u8>> {
    let mut out = prefix(8, cap, key)?;
    out.extend_from_slice(expected);
    Ok(out)
}
