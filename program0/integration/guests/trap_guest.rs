//! Trap-after-one-byte child: verifies software trap recovery in P0, with no
//! engine budget or native lock involved.
#[link(wasm_import_module = "p0")]
unsafe extern "C" {
    fn invoke(ctx: i32, op: i32, value: i32) -> i64;
}
#[unsafe(no_mangle)]
pub extern "C" fn _start() {
    unsafe {
        let _ = invoke(0, 0, 0);
        core::arch::wasm32::unreachable();
    }
}
