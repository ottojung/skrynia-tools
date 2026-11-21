#[link(wasm_import_module = "skrynia")]
unsafe extern "C" {
    fn eval(code: u32, code_len: u32, input_fd: i32, output_fd: i32, cb_idx: i32) -> i32;
}

/// Evaluate a child module as immediate parent mediator using kernel nested-eval ABI.
///
/// Kernel ABI (compose-eval/ccbbbdf): fd-based `(in_fd, out_fd)` stream transfer,
/// callback via indirect function table index (`cb_idx >= 0`) with signature
/// `(req_ptr, req_len, res_ptr, res_cap) -> i32` and immediate-provider mediation.
///
/// BLOCKED (ABI/guest-fd capability): cb-guest P1 has no kernel primitive to create
/// buffer-backed in/out stream FDs for byte-buffer Invoke input/output. P1 cannot
/// repurpose consumed service request/response streams as child I/O channels without
/// breaking service framing. A generic guest-accessible stream/buffer FD creation or
/// buffer-eval mediation ABI must be coordinated with kernel/reviewer.
///
/// Until that bridge exists, buffered registry Invoke cannot be executed here; this
/// returns ERR_PERM (-4) rather than silently misrouting I/O or escalating authority.
pub fn eval_buffered(wasm: &[u8], _input: &[u8]) -> Result<Vec<u8>, i32> {
    let _ = wasm;
    // Cannot fabricate child stream FDs in current guest capability set.
    Err(-4)
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
