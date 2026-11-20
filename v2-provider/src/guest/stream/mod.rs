mod read;
mod write;
pub use read::receive;
pub use write::send;
#[link(wasm_import_module = "skrynia")]
unsafe extern "C" {
    pub(super) fn read(fd: i32, dst: u32, len: u32) -> i32;
    pub(super) fn write(fd: i32, ptr: u32, len: u32) -> i32;
}
