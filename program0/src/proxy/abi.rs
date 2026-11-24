#[cfg(all(test, not(target_arch = "wasm32")))]
pub use super::{abi_stubs::*, abi_stubs2::*};

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "skrynia")]
unsafe extern "C" {
    pub fn open_disk(path: u32, len: u32, mode: u32) -> i32;
    pub fn read(fd: i32, dst: u32, len: u32) -> i32;
    pub fn write(fd: i32, ptr: u32, len: u32) -> i32;
    pub fn close(fd: i32) -> i32;
    pub fn mkdir(path: u32, len: u32, flags: u32) -> i32;
    pub fn rename(old: u32, old_len: u32, new: u32, new_len: u32) -> i32;
    pub fn delete(path: u32, len: u32) -> i32;
    pub fn exists(path: u32, len: u32) -> i32;
    pub fn open_directory(path: u32, len: u32) -> i32;
    pub fn next_entry(fd: i32, dst: u32, cap: u32) -> i32;
    pub fn sync(fd: i32) -> i32;
    pub fn secure_random_bits(dst: u32, bits: u32) -> i32;
    #[allow(dead_code)]
    pub fn eval(code: u32, code_len: u32, input_fd: i32, output_fd: i32, callback: i32) -> i32;
    pub fn eval_delegate(
        code: u32,
        code_len: u32,
        input_fd: i32,
        output_fd: i32,
        callback: i32,
    ) -> i32;
    pub fn try_mutation_lock() -> i32;
    pub fn release_mutation_lock() -> i32;
    pub fn fs_delegate_scope(root: i32, path_ptr: u32, path_len: u32, ops: u32) -> i32;
}
