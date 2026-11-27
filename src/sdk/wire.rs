//! Deterministic typed integer wire codec. Version 1 with a backward-compatible
//! extended count: one byte up to 254 values, `0xff` + LE u32 from 255.
use super::{TypedValue, WireError};

pub const VERSION: u8 = 0x01;
pub const TAG_I32: u8 = 0x32;
pub const TAG_I64: u8 = 0x40;
pub const COUNT_EXTENDED: u8 = 0xff;
/// Practical ceiling for one frame (64 MiB of tagged values).
pub const MAX_FRAME: usize = 64 * 1024 * 1024;

/// Exact encoded length: header plus tagged values, with checked arithmetic.
pub(super) fn frame_len(
    count: usize,
    mut widths: impl Iterator<Item = usize>,
) -> Result<usize, WireError> {
    let header = 2 + 4 * usize::from(count >= usize::from(COUNT_EXTENDED));
    let total = widths.try_fold(header, |sum, width| {
        sum.checked_add(width).ok_or(WireError::TooLarge)
    })?;
    if total > MAX_FRAME {
        return Err(WireError::TooLarge);
    }
    Ok(total)
}

pub fn encode(values: &[TypedValue]) -> Result<Vec<u8>, WireError> {
    let count = values.len();
    if count > u32::MAX as usize {
        return Err(WireError::TooLarge);
    }
    let widths = values.iter().map(|v| match v {
        TypedValue::I32(_) => 5,
        TypedValue::I64(_) => 9,
    });
    let mut out = Vec::with_capacity(frame_len(count, widths)?);
    out.push(VERSION);
    if count < usize::from(COUNT_EXTENDED) {
        out.push(count as u8);
    } else {
        out.push(COUNT_EXTENDED);
        out.extend_from_slice(&(count as u32).to_le_bytes());
    }
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
