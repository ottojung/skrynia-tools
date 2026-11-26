//! The evaluator-supplied `invoke` function, the guest's only import.
//!
//! Import module/name is a userland convention (`p0.invoke`); the kernel
//! neither prescribes nor injects it. No `skrynia` module import remains in
//! this crate's guest build, and no linear-memory pointer crosses stores.

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "p0")]
unsafe extern "C" {
    fn invoke(ctx: i32, op: i32, value: i32) -> i64;
}

pub struct Real;

impl crate::gateway::Transport for Real {
    fn invoke(&self, ctx: i32, op: i32, value: i32) -> i64 {
        #[cfg(target_arch = "wasm32")]
        {
            unsafe { invoke(ctx, op, value) }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = (ctx, op, value);
            crate::gateway::ops::ERR_INVAL
        }
    }
}
