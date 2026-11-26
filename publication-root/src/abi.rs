//! Owner-approved target ABI: byte-only I/O and u64 shared-RAM arguments.
//!
//! Every fixed import below is one of the 19 kernel-registered names with its
//! exact ordered integer type. No bulk read/write and no caller-linear-memory
//! pointer crosses the host boundary.

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "skrynia")]
unsafe extern "C" {
    pub fn open_ram() -> i32;
    pub fn open_disk(ptr: u64, len: u64, mode: u32) -> i32;
    pub fn close(fd: i32) -> i32;
    pub fn read1(fd: i32) -> i32;
    pub fn write1(fd: i32, byte: u32) -> i32;
    pub fn seek(fd: i32, shift: i64) -> i64;
    pub fn truncate(fd: i32, size: u64) -> i32;
    pub fn cas1(fd: i32, off: u64, exp: u32, des: u32) -> i32;
    pub fn mkdir(ptr: u64, len: u64, mode: u32) -> i32;
    pub fn delete(ptr: u64, len: u64) -> i32;
    pub fn sync(fd: i32) -> i32;
    pub fn secure_random_bits(ptr: u64, bits: u64) -> i32;
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
