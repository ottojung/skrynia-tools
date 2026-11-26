#[cfg(target_arch = "wasm32")]
mod inner {
    /// Opaque context handle supplied by P1; never a pointer or table index.
    pub const CTX: i32 = 1;
    pub const GET: i32 = 0;
    pub const PUT: i32 = 1;
    pub const CB_GET: i32 = 3;
    pub const CB_PUT: i32 = 4;
    pub const CB_FLUSH: i32 = 6;

    #[link(wasm_import_module = "p0")]
    unsafe extern "C" {
        fn invoke(ctx: i32, op: i32, value: i32) -> i64;
    }

    pub fn call(op: i32, value: i32) -> i64 {
        unsafe { invoke(CTX, op, value) }
    }

    pub fn byte(value: i64) -> Result<u8, i32> {
        if !(0..=255).contains(&value) {
            return Err(value as i32);
        }
        Ok(value as u8)
    }
}

#[cfg(target_arch = "wasm32")]
pub use inner::{CB_FLUSH, CB_GET, CB_PUT, CTX, GET, PUT, byte, call};
