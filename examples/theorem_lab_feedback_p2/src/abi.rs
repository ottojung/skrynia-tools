//! Target-ABI guest transport: one supplied `p0.invoke` and no legacy imports.
//!
//! Input/output use the P1-owned GET/PUT byte streams. Opaque application
//! requests use the generic callback frame: CB_PUT appends one request byte,
//! CB_FLUSH executes exactly one complete opaque request through the
//! P1-owned callback, CB_GET drains one reply byte (-1 at EOF). Any negative
//! status is final: sticky errors prevent replay.
#[cfg(target_arch = "wasm32")]
mod inner {
    use alloc::vec::Vec;

    #[link(wasm_import_module = "p0")]
    unsafe extern "C" {
        fn invoke(ctx: i32, op: i32, value: i32) -> i64;
    }

    const GET: i32 = 0;
    const PUT: i32 = 1;
    const CB_GET: i32 = 3;
    const CB_PUT: i32 = 4;
    const CB_FLUSH: i32 = 6;
    const EOF: i64 = -1;

    fn call(op: i32, value: i32) -> Result<u8, i32> {
        match unsafe { invoke(0, op, value) } {
            EOF => Err(-1),
            code if code < 0 => Err(code as i32),
            code if code > 255 => Err(-6),
            code => Ok(code as u8),
        }
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

    pub fn invoke_raw(req: &[u8], out: &mut [u8]) -> i32 {
        for &byte in req {
            if call(CB_PUT, i32::from(byte)).is_err() {
                return -3;
            }
        }
        if unsafe { invoke(0, CB_FLUSH, 0) } < 0 {
            return -5;
        }
        let mut len: usize = 0;
        loop {
            match call(CB_GET, 0) {
                Err(-1) => return len as i32,
                Err(code) => return code,
                Ok(byte) if len >= out.len() => return -6,
                Ok(byte) => {
                    out[len] = byte;
                    len += 1;
                }
            }
        }
    }
}
#[cfg(target_arch = "wasm32")]
pub use inner::*;
