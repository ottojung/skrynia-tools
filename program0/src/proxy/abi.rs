#[link(wasm_import_module = "skrynia")]
unsafe extern "C" {
    pub fn open_disk(path: u32, len: u32, mode: u32) -> i32;
    pub fn read(fd: i32, dst: u32, len: u32) -> i32;
    pub fn write(fd: i32, ptr: u32, len: u32) -> i32;
    pub fn close(fd: i32) -> i32;
    pub fn seek(fd: i32, off: u64) -> i32;
    pub fn truncate(fd: i32, size: u64) -> i32;
    pub fn sync(fd: i32) -> i32;
    pub fn rename(a: u32, al: u32, b: u32, bl: u32) -> i32;
    pub fn delete(path: u32, len: u32) -> i32;
    pub fn mkdir(path: u32, len: u32, mode: u32) -> i32;
    pub fn exists(path: u32, len: u32) -> i32;
    pub fn is_dir(path: u32, len: u32) -> i32;
    pub fn open_directory(path: u32, len: u32) -> i32;
    pub fn next_entry(fd: i32, dst: u32, cap: u32) -> i32;
    pub fn secure_random_bits(ptr: u32, bits: u32) -> i32;
    pub fn open_ram() -> i32;
    pub fn cas_ram_u64(fd: i32, off: u64, exp: u64, des: u64, op: u32) -> i32;
    pub fn eval(code: u32, code_len: u32, input_fd: i32, output_fd: i32, cb: i32) -> i32;
}
