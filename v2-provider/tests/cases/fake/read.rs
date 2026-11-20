use super::{Fake, InvokeError};
pub fn call(this: &Fake, tail: &[u8]) -> Result<Vec<u8>, InvokeError> {
    if tail.len() != 8 || i32::from_le_bytes(tail[..4].try_into().unwrap()) != 99 {
        return Err(InvokeError::Unavailable);
    }
    let mut state = this.state.borrow_mut();
    if !state.0 {
        return Err(InvokeError::Unavailable);
    };
    let bytes = b"abcdef";
    let n = (bytes.len() - state.1).min(u32::from_le_bytes(tail[4..].try_into().unwrap()) as usize);
    let mut out = (n as i32).to_le_bytes().to_vec();
    out.extend_from_slice(&bytes[state.1..state.1 + n]);
    state.1 += n;
    Ok(out)
}
