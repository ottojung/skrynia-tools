use super::{Fake, InvokeError};
pub fn call(this: &Fake, tail: &[u8]) -> Result<Vec<u8>, InvokeError> {
    if tail != 99i32.to_le_bytes() {
        return Err(InvokeError::Unavailable);
    }
    this.state.replace((false, 0));
    Ok(0i32.to_le_bytes().to_vec())
}
