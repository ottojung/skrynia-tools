use super::{Error, Result};
use serde_json::Value;

pub(super) fn hex32(v: &Value, key: &str) -> Result<[u8; 32]> {
    let bytes = hex_bytes(super::field::str_field(v, key)?)?;
    bytes.try_into().map_err(|_| Error::Config("config-hex"))
}

pub(super) fn hex16(v: &Value, key: &str) -> Result<[u8; 16]> {
    let bytes = hex_bytes(super::field::str_field(v, key)?)?;
    bytes.try_into().map_err(|_| Error::Config("config-hex"))
}

fn hex_bytes(s: &str) -> Result<Vec<u8>> {
    if !s.len().is_multiple_of(2) {
        return Err(Error::Config("config-hex"));
    }
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len() / 2);
    for i in (0..b.len()).step_by(2) {
        out.push(nibble(b[i])? << 4 | nibble(b[i + 1])?);
    }
    Ok(out)
}

fn nibble(c: u8) -> Result<u8> {
    match c {
        b'0'..=b'9' => Ok(c - b'0'),
        b'a'..=b'f' => Ok(c - b'a' + 10),
        b'A'..=b'F' => Ok(c - b'A' + 10),
        _ => Err(Error::Config("config-hex")),
    }
}
