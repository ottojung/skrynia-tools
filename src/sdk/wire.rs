//! Deterministic typed integer wire codec, version 1 (K0 wire choice).
use super::typed::ValType;

pub const VERSION: u8 = 0x01;
pub const TAG_I32: u8 = 0x32;
pub const TAG_I64: u8 = 0x40;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TypedValue {
    I32(i32),
    I64(i64),
}

impl TypedValue {
    pub fn kind(self) -> ValType {
        match self {
            Self::I32(_) => ValType::I32,
            Self::I64(_) => ValType::I64,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WireError {
    Version,
    Count,
    Tag,
    Truncated,
    Trailing,
}

pub(super) fn word(bytes: &[u8], at: usize, len: usize) -> Result<u64, WireError> {
    let slice = bytes.get(at..at + len).ok_or(WireError::Truncated)?;
    let mut buf = [0u8; 8];
    buf[..len].copy_from_slice(slice);
    Ok(u64::from_le_bytes(buf))
}

/// byte0=1, byte1=N, then N tagged little-endian integers.
pub fn encode(values: &[TypedValue]) -> Result<Vec<u8>, WireError> {
    if values.len() > usize::from(u8::MAX) {
        return Err(WireError::Count);
    }
    let mut out = vec![VERSION, values.len() as u8];
    for value in values {
        match value {
            TypedValue::I32(raw) => {
                out.push(TAG_I32);
                out.extend_from_slice(&raw.to_le_bytes());
            }
            TypedValue::I64(raw) => {
                out.push(TAG_I64);
                out.extend_from_slice(&raw.to_le_bytes());
            }
        }
    }
    Ok(out)
}
