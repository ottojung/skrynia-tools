use crate::{Callback, InvokeError};

#[link(wasm_import_module = "skrynia")]
unsafe extern "C" {
    fn invoke(req: u32, len: u32, output: u32, capacity: u32) -> i32;
}
pub struct Host;
impl Callback for Host {
    fn invoke(&self, bytes: &[u8]) -> Result<Vec<u8>, InvokeError> {
        // Generic syscall callbacks return at most 64KiB plus a signed status.
        let mut out = vec![0u8; 64 * 1024 + 4];
        let n = unsafe {
            invoke(
                bytes.as_ptr() as u32,
                bytes.len() as u32,
                out.as_mut_ptr() as u32,
                out.len() as u32,
            )
        };
        if n < 0 || n as usize > out.len() {
            return Err(InvokeError::Unavailable);
        }
        out.truncate(n as usize);
        Ok(out)
    }
}
