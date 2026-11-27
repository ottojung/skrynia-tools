//! Dumb operator-token verifier. P0 reads only the 33-byte frame header; all
//! remaining request bytes stay opaque and are consumed by the guest.
use super::{abi, alloc, code, token};

pub const EOF: i32 = -1;
pub const DENIED: i32 = -4;
pub const VERSION: i32 = 1;
pub const TOKEN_LEN: usize = 32;
pub const VERIFIER_PATH: &[u8] = b"operator/security/program0.sha256";

pub fn require(input_fd: i32, ram_fd: i32, pos: &mut u64, base: u64) -> Result<(), i32> {
    if unsafe { abi::read1(input_fd) } != VERSION {
        return Err(DENIED);
    }
    let mut raw = [0u8; TOKEN_LEN];
    for byte in &mut raw {
        let b = unsafe { abi::read1(input_fd) };
        if !(0..=255).contains(&b) {
            return Err(DENIED);
        }
        *byte = b as u8;
    }
    code::put_path(ram_fd, pos, base, VERIFIER_PATH)?;
    let fd = unsafe { abi::open_disk(base + alloc::SLOT_PATH_AT, VERIFIER_PATH.len() as u64, 1) };
    if fd < 0 {
        return Err(DENIED);
    }
    let mut expected = [0u8; TOKEN_LEN];
    for byte in &mut expected {
        let b = unsafe { abi::read1(fd) };
        if !(0..=255).contains(&b) {
            unsafe { abi::close(fd) };
            return Err(DENIED);
        }
        *byte = b as u8;
    }
    let trailing = unsafe { abi::read1(fd) };
    unsafe { abi::close(fd) };
    if trailing != EOF || !token::matches(&raw, &expected) {
        return Err(DENIED);
    }
    Ok(())
}
