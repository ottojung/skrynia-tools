//! Deliberately wrong import type: must be rejected before execution.
#[link(wasm_import_module = "p0")]
unsafe extern "C" {
    fn invoke(ctx: i32, op: i32, value: i32) -> i32;
}
#[unsafe(no_mangle)]
pub extern "C" fn _start() {
    unsafe { invoke(1, 0, 0); }
}
