//! Request encoding matching native-service `[VERSION, op, ...]` wire.
use super::{Error, Result};

pub const VERSION: u8 = 1;
pub const OP_HINT_LIST: u8 = 20;
pub const OP_HINT_READ: u8 = 21;
pub const OP_HINT_ACK: u8 = 22;
pub const OP_SUB_FETCH: u8 = 23;

fn id(out: &mut Vec<u8>, value: &[u8]) -> Result<()> {
    let len = u16::try_from(value.len()).map_err(|_| Error::Config("id"))?;
    out.extend_from_slice(&len.to_le_bytes());
    out.extend_from_slice(value);
    Ok(())
}

fn blob(out: &mut Vec<u8>, value: &[u8]) -> Result<()> {
    let len = u32::try_from(value.len()).map_err(|_| Error::Config("blob"))?;
    out.extend_from_slice(&len.to_le_bytes());
    out.extend_from_slice(value);
    Ok(())
}

fn head(out: &mut Vec<u8>, op: u8, cap: &[u8], ns: &[u8], key: &[u8]) -> Result<()> {
    out.push(VERSION);
    out.push(op);
    id(out, cap)?;
    id(out, ns)?;
    blob(out, key)
}

pub fn hint_list(cap: &[u8], ns: &[u8], key: &[u8]) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    head(&mut out, OP_HINT_LIST, cap, ns, key)?;
    Ok(out)
}

pub fn hint_read(cap: &[u8], ns: &[u8], key: &[u8], seq: u64) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    head(&mut out, OP_HINT_READ, cap, ns, key)?;
    out.extend_from_slice(&seq.to_le_bytes());
    Ok(out)
}

pub fn hint_ack(cap: &[u8], ns: &[u8], key: &[u8], seq: u64) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    head(&mut out, OP_HINT_ACK, cap, ns, key)?;
    out.extend_from_slice(&seq.to_le_bytes());
    Ok(out)
}

pub fn sub_fetch(cap: &[u8], ns: &[u8], key: &[u8]) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    head(&mut out, OP_SUB_FETCH, cap, ns, key)?;
    Ok(out)
}
