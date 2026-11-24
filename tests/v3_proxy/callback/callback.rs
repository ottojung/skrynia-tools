#[link(wasm_import_module = "skrynia")]
unsafe extern "C" {
    fn invoke(req: u32, len: u32, dst: u32, cap: u32) -> i32;
}
pub fn call(req: &[u8], out: &mut [u8]) -> i32 {
    unsafe {
        invoke(
            req.as_ptr() as u32,
            req.len() as u32,
            out.as_mut_ptr() as u32,
            out.len() as u32,
        )
    }
}
