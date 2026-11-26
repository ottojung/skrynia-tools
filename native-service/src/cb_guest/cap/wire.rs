pub const OPEN_DISK: u8 = 1;
pub const READ: u8 = 2;
pub const WRITE: u8 = 3;
pub const CLOSE: u8 = 4;
pub const MKDIR: u8 = 5;
pub const DELETE: u8 = 6;
pub const RENAME: u8 = 7;
pub const OPEN_DIR: u8 = 8;
pub const NEXT_ENTRY: u8 = 9;
pub const SYNC: u8 = 10;
pub const RANDOM: u8 = 11;
pub const TRY_LOCK: u8 = 12;
pub const RELEASE_LOCK: u8 = 13;
/// Kernel's explicit ENOENT: only a genuine missing object maps here.
pub const E_NOENT: i32 = crate::gateway::codes::E_NOENT;
/// EBADF, EIO, ENOTSUP and EINVAL are failures, never absence.
pub fn is_missing(code: i32) -> bool {
    crate::gateway::codes::is_missing(code)
}
pub fn put_u32(v: &mut Vec<u8>, n: u32) {
    v.extend_from_slice(&n.to_le_bytes());
}
pub fn put_bytes(v: &mut Vec<u8>, b: &[u8]) {
    put_u32(v, b.len() as u32);
    v.extend_from_slice(b);
}
