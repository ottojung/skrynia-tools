use super::{Fake, InvokeError};
pub fn call(this: &Fake, tail: &[u8]) -> Result<Vec<u8>, InvokeError> {
    let Some(len) = tail.get(..4) else {
        return Err(InvokeError::Unavailable);
    };
    let n = u32::from_le_bytes(len.try_into().unwrap()) as usize;
    let Some(path) = tail.get(4..4 + n) else {
        return Err(InvokeError::Unavailable);
    };
    if tail.get(4 + n..) != Some(&1u32.to_le_bytes()) {
        return Err(InvokeError::Unavailable);
    }
    if path == b"v2/objects/public/missing.txt" {
        return Ok((-2i32).to_le_bytes().to_vec());
    }
    if path != b"v2/objects/public/file.txt" {
        return Err(InvokeError::Unavailable);
    }
    this.state.replace((true, 0));
    Ok((99i32).to_le_bytes().to_vec())
}
