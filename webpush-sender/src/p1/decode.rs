//! Decode native-service `encode_res`: version, status, u32 LE length, body.
use super::{Error, Result};

pub fn decode(src: &[u8]) -> Result<Vec<u8>> {
    if src.len() < 6 {
        return Err(Error::Wire("short"));
    }
    if src[0] != 1 {
        return Err(Error::Wire("version"));
    }
    let len = u32::from_le_bytes(src[2..6].try_into().map_err(|_| Error::Wire("len"))?);
    let len = usize::try_from(len).map_err(|_| Error::Wire("len"))?;
    if src.len() != 6 + len {
        return Err(Error::Wire("len"));
    }
    let body = src[6..].to_vec();
    match src[1] {
        0 => Ok(body),
        1 if !body.is_empty() => Err(Error::P1(body[0])),
        _ => Err(Error::Wire("status")),
    }
}
