//! The token verifier reads only the header; payload stays opaque.
use super::{abi, token};

pub const EOF: i32 = -1;
pub const DENIED: i32 = -4;
pub const VERSION: i32 = 1;
pub const TOKEN_LEN: usize = 32;
pub const VERIFIER_PATH: &[u8] = b"operator/security/program0.sha256";

fn exact(fd: i32, bytes: &mut [u8]) -> Result<(), i32> {
    let mut pos = 0;
    while pos < bytes.len() {
        let dst = &mut bytes[pos..];
        let n = unsafe { abi::read(fd, dst.as_mut_ptr() as u64, dst.len() as u64) };
        if n <= 0 || n as usize > dst.len() {
            return Err(DENIED);
        }
        pos += n as usize;
    }
    Ok(())
}

pub fn require(input_fd: i32) -> Result<(), i32> {
    if unsafe { abi::read1(input_fd) } != VERSION {
        return Err(DENIED);
    }
    let mut raw = [0u8; TOKEN_LEN];
    exact(input_fd, &mut raw)?;
    let fd =
        unsafe { abi::open_disk(VERIFIER_PATH.as_ptr() as u64, VERIFIER_PATH.len() as u64, 1) };
    if fd < 0 {
        return Err(DENIED);
    }
    let mut expected = [0u8; TOKEN_LEN];
    let result = exact(fd, &mut expected);
    let trailing = unsafe { abi::read1(fd) };
    unsafe { abi::close(fd) };
    if result.is_err() || trailing != EOF || !token::matches(&raw, &expected) {
        return Err(DENIED);
    }
    Ok(())
}
