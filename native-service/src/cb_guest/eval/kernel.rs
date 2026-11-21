#[link(wasm_import_module = "skrynia")]
unsafe extern "C" {
    fn eval_buffered(cp: u32, cl: u32, rp: u32, rl: u32, dp: u32, dc: u32, cb_idx: i32) -> i32;
}

const RES_CAP: usize = 8 * 1024 * 1024;

/// Evaluate registered immutable code as an ordinary child guest.
///
/// Kernel buffered-eval ABI creates bounded child request/response streams,
/// so no numeric root FD is granted to this guest. The callback index routes
/// child appdata requests back to the app-scoped mediator in `cb.rs`.
pub fn eval_buffered_call(wasm: &[u8], input: &[u8]) -> Result<Vec<u8>, i32> {
    let cb_idx = super::cb::callback_index();
    let mut out = vec![0u8; RES_CAP];
    let n = unsafe {
        eval_buffered(
            wasm.as_ptr() as u32,
            wasm.len() as u32,
            input.as_ptr() as u32,
            input.len() as u32,
            out.as_mut_ptr() as u32,
            out.len() as u32,
            cb_idx,
        )
    };
    if n < 0 {
        return Err(n);
    }
    let n = n as usize;
    if n > out.len() {
        return Err(-11);
    }
    out.truncate(n);
    Ok(out)
}
