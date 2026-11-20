mod read;
use super::{abi, token};
const TOKEN_BYTES: usize = 32;
const VERSION: u8 = 1;
const VERIFIER_PATH: &[u8] = b"security/program0.sha256";
const DENIED: i32 = -4;

pub fn require(input_fd: i32) -> Result<(), i32> {
    let mut header = [0u8; 1 + TOKEN_BYTES];
    read::exact(input_fd, &mut header)?;
    if header[0] != VERSION {
        return Err(DENIED);
    }
    let expected = read::verifier()?;
    let supplied: &[u8; TOKEN_BYTES] = header[1..].try_into().map_err(|_| DENIED)?;
    if token::matches(supplied, &expected) {
        Ok(())
    } else {
        Err(DENIED)
    }
}
