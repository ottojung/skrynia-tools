#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "skrynia")]
unsafe extern "C" {
    pub fn open_disk(path: u32, len: u32, mode: u32) -> i32;
    pub fn read(fd: i32, dst: u32, len: u32) -> i32;
    pub fn write(fd: i32, src: u32, len: u32) -> i32;
    pub fn close(fd: i32) -> i32;
    pub fn sync(fd: i32) -> i32;
    pub fn mkdir(path: u32, len: u32, mode: u32) -> i32;
    pub fn delete(path: u32, len: u32) -> i32;
    pub fn secure_random_bits(dst: u32, bits: u32) -> i32;
}

// Only the root program may import this namespace; the kernel rejects it in
// every evaluated child. Argument and result are one u32 bit pattern each.
#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "syscall")]
unsafe extern "C" {
    pub fn publish(ticket: u32) -> i32;
    pub fn unpublish(ticket: u32) -> i32;
}
