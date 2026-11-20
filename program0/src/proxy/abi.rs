#[link(wasm_import_module = "skrynia")]
unsafe extern "C" {
    pub fn open_disk(path: u32, len: u32, mode: u32) -> i32;
    pub fn read(fd: i32, dst: u32, len: u32) -> i32;
    pub fn write(fd: i32, ptr: u32, len: u32) -> i32;
    pub fn close(fd: i32) -> i32;
    pub fn eval(code: u32, code_len: u32, input_fd: i32, output_fd: i32, callback: i32) -> i32;
}
