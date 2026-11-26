//! Generic kernel-operation framing over the supplied byte-stream gateway.

pub fn raw(req: &[u8], cap: usize) -> Result<Vec<u8>, i32> {
    crate::gateway::stream::call(
        &crate::cb_guest::gateway::Real,
        crate::gateway::ops::CTX_MAIN,
        req,
        cap.max(8),
    )
    .map_err(|e| e as i32)
}

pub fn call(req: &[u8]) -> Result<(i32, Vec<u8>), i32> {
    call_with_capacity(req, 256)
}

pub fn call_with_capacity(req: &[u8], capacity: usize) -> Result<(i32, Vec<u8>), i32> {
    let res = raw(req, capacity)?;
    if res.len() < 4 {
        return Err(-1);
    }
    let code = i32::from_le_bytes([res[0], res[1], res[2], res[3]]);
    Ok((code, res[4..].to_vec()))
}
