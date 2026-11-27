//! ABI 3.1: buffer addresses are u64 offsets into this instance's memory.
#[cfg(not(target_arch = "wasm32"))]
pub use super::fake::*;

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "skrynia_v3_1")]
unsafe extern "C" {
    pub fn open_ram() -> i32;
    pub fn open_disk(path: u64, len: u64, mode: i32) -> i32;
    pub fn exists(path: u64, len: u64) -> i32;
    pub fn close(fd: i32) -> i32;
    pub fn read(fd: i32, dst: u64, len: u64) -> i64;
    pub fn write(fd: i32, src: u64, len: u64) -> i64;
    pub fn read1(fd: i32) -> i32;
    pub fn seek(fd: i32, shift: i64) -> i64;
    pub fn truncate(fd: i32, size: u64) -> i32;
    pub fn cas1(fd: i32, offset: u64, expected: i32, desired: i32) -> i32;
    pub fn eval(code: u64, size: u64, function: i32, arg0: i32, arg1: i32) -> i64;
    pub fn secure_random_bits(dst: u64, bits: u64) -> i32;
    pub fn mkdir(path: u64, len: u64, mode: i32) -> i32;
    pub fn delete(path: u64, len: u64) -> i32;
    pub fn rename(old: u64, old_len: u64, new: u64, new_len: u64) -> i32;
    pub fn open_directory(path: u64, len: u64) -> i32;
    pub fn next_entry(fd: i32, dst: u64, cap: u64) -> i32;
    pub fn sync(fd: i32) -> i32;
}
