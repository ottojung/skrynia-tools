//! Strict decoder: signature mismatch or malformed frame fails before execution.
use super::typed::ValType;
use super::wire::{TAG_I32, TAG_I64, TypedValue, VERSION, WireError, word};

/// Decode a response against the exact declared result types.
///
/// Empty result lists are valid: [VERSION, 0]. No replay is performed here;
/// a failed call is final for this codec and the caller must not retry it.
pub fn decode(bytes: &[u8], expected: &[ValType]) -> Result<Vec<TypedValue>, WireError> {
    if bytes.first().copied() != Some(VERSION) {
        return Err(WireError::Version);
    }
    let count = *bytes.get(1).ok_or(WireError::Truncated)?;
    if count != expected.len() as u8 {
        return Err(WireError::Count);
    }
    let mut at = 2;
    let mut out = Vec::with_capacity(expected.len());
    for kind in expected {
        let tag = *bytes.get(at).ok_or(WireError::Truncated)?;
        at += 1;
        match kind {
            ValType::I32 => {
                if tag != TAG_I32 {
                    return Err(WireError::Tag);
                }
                out.push(TypedValue::I32(word(bytes, at, 4)? as i32));
                at += 4;
            }
            ValType::I64 => {
                if tag != TAG_I64 {
                    return Err(WireError::Tag);
                }
                out.push(TypedValue::I64(word(bytes, at, 8)? as i64));
                at += 8;
            }
        }
    }
    if at != bytes.len() {
        return Err(WireError::Trailing);
    }
    Ok(out)
}
