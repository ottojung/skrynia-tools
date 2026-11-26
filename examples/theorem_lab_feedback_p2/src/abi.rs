//! Target-ABI guest transport: one supplied `p0.invoke` and no legacy imports.
//!
//! Input/output use the P1-owned GET/PUT byte streams. Application requests
//! use the generic callback frame implemented in `abi/callback_frame.rs`.
#[cfg(target_arch = "wasm32")]
mod inner {
    use alloc::vec::Vec;

    #[link(wasm_import_module = "p0")]
    unsafe extern "C" {
        fn invoke(ctx: i32, op: i32, value: i32) -> i64;
    }

    pub const GET: i32 = 0;
    pub const PUT: i32 = 1;
    pub const CB_GET: i32 = 3;
    pub const CB_PUT: i32 = 4;
    pub const CB_FLUSH: i32 = 6;
    pub const EOF: i64 = -1;

    pub fn call(op: i32, value: i32) -> Result<u8, i32> {
        match unsafe { invoke(0, op, value) } {
            EOF => Err(-1),
            code if code < 0 => Err(code as i32),
            code if code > 255 => Err(-6),
            code => Ok(code as u8),
        }
    }

    pub fn flush(op: i32) -> bool {
        (unsafe { invoke(0, op, 0) }) >= 0
    }

    pub fn read_all(_fd: i32) -> Vec<u8> {
        let mut out = Vec::new();
        while let Ok(byte) = call(GET, 0) {
            out.push(byte);
        }
        out
    }

    pub fn write_all(_fd: i32, bytes: &[u8]) {
        for &byte in bytes {
            if call(PUT, i32::from(byte)).is_err() {
                return;
            }
        }
    }
}
#[cfg(target_arch = "wasm32")]
mod callback_frame;
#[cfg(target_arch = "wasm32")]
pub use callback_frame::invoke_raw;
#[cfg(target_arch = "wasm32")]
pub use inner::{read_all, write_all};
