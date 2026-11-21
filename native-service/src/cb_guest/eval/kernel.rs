#[link(wasm_import_module = "skrynia")]
unsafe extern "C" {
    fn eval(code: u32, code_len: u32, input_fd: i32, output_fd: i32, cb_idx: i32) -> i32;
    fn eval_buffered(cp: u32, cl: u32, rp: u32, rl: u32, dp: u32, dc: u32, cb_idx: i32) -> i32;
}

const RES_CAP: usize = 8 * 1024 * 1024;

/// Evaluate a child module as immediate parent mediator using kernel buffered-eval ABI.
///
/// Kernel ABI (004c01b, docs/buffered-eval-abi.md, atop nested mediation 7695fd3/6bc52ec):
/// `eval_buffered(code_ptr, code_len, req_ptr, req_len, res_ptr, res_cap, cb_idx) -> i32`.
/// Kernel creates bounded request/response stream FDs for the leaf child `(in_fd,out_fd)`,
/// delivers callbacks to the immediate parent provider via `cb_idx` (indirect table), and
/// copies child output to `res_ptr` on success (returns byte count).
///
/// `cb_idx` is always provided so P2 may request mediated operations (e.g. appdata) under
/// per-invoke, app-scoped delegated rights enforced by `cb::skrynia_mediator_cb`.
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

/// Fd-streaming mediated eval for contexts where caller supplies child I/O fds.
/// Returns child's eval result code; child I/O and callback mediation occur via kernel.
#[allow(dead_code)]
pub fn eval_fds(wasm: &[u8], in_fd: i32, out_fd: i32, with_cb: bool) -> i32 {
    let cb_idx = if with_cb {
        super::cb::callback_index()
    } else {
        -1
    };
    unsafe {
        eval(
            wasm.as_ptr() as u32,
            wasm.len() as u32,
            in_fd,
            out_fd,
            cb_idx,
        )
    }
}
