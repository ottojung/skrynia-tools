//! Owner-approved ABI 3.1 (`skrynia_v3_1`): 21 fixed functions.
//!
//! Every buffer pointer and length is `u64` and addresses this instance's
//! ordinary exported linear memory named `memory`. Paths, randomness and bulk
//! descriptor data therefore live in private buffers, never in shared RAM.
//! `read`/`write` return the actual byte count: `0` is EOF / no progress, a
//! short count is valid, and a negative value is an error before progress.
//! `read1`, `write1`, `seek` and `cas1` keep their unchanged scalar semantics.

#[cfg(target_arch = "wasm32")]
#[allow(dead_code)]
#[link(wasm_import_module = "skrynia_v3_1")]
unsafe extern "C" {
    pub fn open_ram() -> i32;
    pub fn open_disk(path: u64, len: u64, mode: u32) -> i32;
    pub fn close(fd: i32) -> i32;
    pub fn read(fd: i32, dst: u64, len: u64) -> i64;
    pub fn write(fd: i32, src: u64, len: u64) -> i64;
    pub fn read1(fd: i32) -> i32;
    pub fn write1(fd: i32, byte: u32) -> i32;
    pub fn seek(fd: i32, shift: i64) -> i64;
    pub fn truncate(fd: i32, size: u64) -> i32;
    pub fn mkdir(path: u64, len: u64, mode: u32) -> i32;
    pub fn exists(path: u64, len: u64) -> i32;
    pub fn is_dir(path: u64, len: u64) -> i32;
    pub fn delete(path: u64, len: u64) -> i32;
    pub fn sync(fd: i32) -> i32;
    pub fn rename(old: u64, old_len: u64, new: u64, new_len: u64) -> i32;
    pub fn open_directory(path: u64, len: u64) -> i32;
    pub fn next_entry(fd: i32, dst: u64, cap: u64) -> i32;
    pub fn cas1(fd: i32, offset: u64, expected: u32, desired: u32) -> i32;
    pub fn now() -> i64;
    pub fn secure_random_bits(dst: u64, bits: u64) -> i32;
    pub fn eval(code: u64, size: u64, function: i32, arg0: i32, arg1: i32) -> i64;
}

// Only the root program may import this namespace; the kernel rejects it in
// every evaluated child. Both imports keep the typed (i32)->i32 provider ABI:
// the ticket is passed as an i32 bit pattern and the status returns as i32.
#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "syscall")]
unsafe extern "C" {
    pub fn publish(ticket: i32) -> i32;
    pub fn unpublish(ticket: i32) -> i32;
}
