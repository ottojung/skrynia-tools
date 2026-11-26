//! Target-ABI fixed imports used by P0. All buffer addresses and lengths are
//! u64 shared-RAM offsets; no WASM linear-memory pointer crosses this boundary.
#[cfg(not(target_arch = "wasm32"))]
pub use super::fake::*;

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "skrynia")]
unsafe extern "C" {
    pub fn open_ram() -> i32;
    pub fn open_disk(path: u64, len: u64, mode: i32) -> i32;
    pub fn exists(path: u64, len: u64) -> i32;
    pub fn close(fd: i32) -> i32;
    pub fn read1(fd: i32) -> i32;
    pub fn write1(fd: i32, byte: i32) -> i32;
    pub fn seek(fd: i32, shift: i64) -> i64;
    pub fn truncate(fd: i32, size: u64) -> i32;
    pub fn cas1(fd: i32, offset: u64, expected: i32, desired: i32) -> i32;
    pub fn eval(code: u64, size: u64, function: i32, arg0: i32, arg1: i32) -> i64;
    pub fn secure_random_bits(dst: u64, bits: u64) -> i32;
}
