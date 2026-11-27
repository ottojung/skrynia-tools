use super::ExecError;
use crate::decode_status;
use alloc::vec::Vec;
use sha2::{Digest, Sha256};

pub fn hash32(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}

/// Drain the evaluator-supplied input bytes; EOF is -1.
#[cfg(target_arch = "wasm32")]
pub fn read_all(cap: usize) -> Result<Vec<u8>, i32> {
    use crate::{GET, byte, call};
    let mut out = Vec::new();
    loop {
        let value = call(GET, 0);
        if value == -1 {
            return Ok(out);
        }
        if out.len() >= cap {
            return Err(-8);
        }
        out.push(byte(value)?);
    }
}

/// Emit output bytes; the evaluator accumulates them for the native reply.
#[cfg(target_arch = "wasm32")]
pub fn write_all(bytes: &[u8]) -> Result<(), i32> {
    use crate::{PUT, call};
    for &value in bytes {
        if call(PUT, i32::from(value)) < 0 {
            return Err(-8);
        }
    }
    Ok(())
}

pub fn invoke_req<F>(invoke: &mut F, req: &[u8]) -> Result<(u8, Vec<u8>), ExecError>
where
    F: FnMut(&[u8], &mut [u8]) -> i32,
{
    let mut out = alloc::vec![0u8; 1024 * 1024];
    let n = invoke(req, &mut out);
    if n < 0 || n as usize > out.len() {
        return Err(ExecError::Io);
    }
    let (status, body) = decode_status(&out[..n as usize]).map_err(|_| ExecError::Io)?;
    Ok((status, body.to_vec()))
}
