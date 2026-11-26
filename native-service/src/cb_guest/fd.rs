//! Request/response descriptor streams for the ordinary service entrypoint.
//!
//! GET falls through to the authenticated input descriptor. The response is
//! written with the generic framed WRITE op (3, stdout handle, u32 length),
//! not by stuffing raw bytes into PUT: P0 interprets every PUT frame as a
//! kernel operation. No fd number or linear-memory pointer is invented by
//! the guest beyond that shared userland wire convention.

const MAX_REQ: usize = 16 * 1024 * 1024;
const CHUNK: usize = 64 * 1024;

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
    for chunk in data.chunks(CHUNK) {
        let mut frame = vec![crate::cb_guest::cap::wire::WRITE];
        frame.extend_from_slice(&1u32.to_le_bytes());
        frame.extend_from_slice(&(chunk.len() as u32).to_le_bytes());
        frame.extend_from_slice(chunk);
        crate::gateway::stream::call(&real, ctx, &frame, 16).map_err(|e| e as i32)?;
    }
    Ok(())
}
