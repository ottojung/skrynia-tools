//! Strict decoder: signature mismatch or malformed frame fails before execution.
use super::wire::{COUNT_EXTENDED, TAG_I32, TAG_I64, VERSION, frame_len};
use super::{TypedValue, ValType, WireError, word};

fn shape(kind: &ValType) -> (u8, usize) {
    if *kind == ValType::I32 {
        (TAG_I32, 4)
    } else {
        (TAG_I64, 8)
    }
}

fn read_count(bytes: &[u8]) -> Result<(usize, usize), WireError> {
    let first = *bytes.get(1).ok_or(WireError::Truncated)?;
    if first != COUNT_EXTENDED {
        return Ok((first as usize, 2));
    }
    let tail = bytes.get(2..6).ok_or(WireError::Truncated)?;
    let wide = u32::from_le_bytes(tail.try_into().unwrap()) as usize;
    if wide < usize::from(COUNT_EXTENDED) {
        return Err(WireError::NonMinimal);
    }
    Ok((wide, 6))
}

/// Decode a response against the exact declared result types. Empty lists are
/// valid: `[VERSION, 0]`. No replay is performed here.
pub fn decode(bytes: &[u8], expected: &[ValType]) -> Result<Vec<TypedValue>, WireError> {
    if bytes.first().copied() != Some(VERSION) {
        return Err(WireError::Version);
    }
    let (count, header) = read_count(bytes)?;
    if count != expected.len() {
        return Err(WireError::Count);
    }
    let widths = expected.iter().map(|t| shape(t).1 + 1);
    let expected_len = frame_len(count, widths)?;
    let mut at = header;
    let mut out = Vec::with_capacity(count);
    for kind in expected {
        let (tag, size) = shape(kind);
        if *bytes.get(at).ok_or(WireError::Truncated)? != tag {
            return Err(WireError::Tag);
        }
        at += 1;
        out.push(match kind {
            ValType::I32 => TypedValue::I32(word(bytes, at, size)? as i32),
            _ => TypedValue::I64(word(bytes, at, size)? as i64),
        });
        at += size;
    }
    if bytes.len() != expected_len {
        return Err(WireError::Trailing);
    }
    Ok(out)
}
