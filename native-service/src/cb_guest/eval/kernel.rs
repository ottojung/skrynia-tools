//! Nested `_start` guest evaluation through the mediated userland protocol.
//!
//! Code bytes are staged by P0 in its own software-managed shared RAM; no
//! WASM linear-memory pointer and no P1 function-table index is sent to the
//! evaluator. T2's relay opcode 15 carries code only and the child reads its
//! input from the invocation descriptor, so no private input copy is staged
//! here. Callback-dependent guests fail closed with ENOTSUP; they need the
//! in-process userland interpreter (T6) instead.

const RES_CAP: usize = 8 * 1024 * 1024;

pub fn eval_nested(wasm: &[u8], _input: &[u8]) -> Result<Vec<u8>, i32> {
    let mut req = vec![crate::cb_guest::cap::wire::EVAL];
    req.extend_from_slice(&(wasm.len() as u32).to_le_bytes());
    req.extend_from_slice(wasm);
    crate::gateway::eval::call(
        &crate::cb_guest::gateway::Real,
        crate::gateway::ops::CTX_NESTED,
        &req,
        RES_CAP,
    )
    .map_err(|e| e as i32)
}
