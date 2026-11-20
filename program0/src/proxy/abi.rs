#[link(wasm_import_module = "skrynia")]
unsafe extern "C" {
    pub fn run_program1(input_fd: i32, output_fd: i32) -> i32;
}
