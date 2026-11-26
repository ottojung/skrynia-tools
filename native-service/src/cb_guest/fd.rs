//! Request/response descriptor streams for the ordinary service entrypoint.
//!
//! P0 preloads the authenticated request into the main context and drains
//! the response after the single FLUSH. No fd numbers or linear-memory
//! buffers are passed across the evaluator boundary.

const MAX_REQ: usize = 16 * 1024 * 1024;

pub fn read_request() -> Result<Vec<u8>, i32> {
    crate::gateway::stream::recv(
        &crate::cb_guest::gateway::Real,
        crate::gateway::ops::CTX_MAIN,
        MAX_REQ,
    )
    .map_err(|e| e as i32)
}

pub fn write_response(data: &[u8]) -> Result<(), i32> {
    let real = crate::cb_guest::gateway::Real;
    let ctx = crate::gateway::ops::CTX_MAIN;
    crate::gateway::stream::send(&real, ctx, data).map_err(|e| e as i32)?;
    crate::gateway::stream::flush(&real, ctx).map_err(|e| e as i32)
}
