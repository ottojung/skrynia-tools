use crate::{Callback, InvokeError};

#[link(wasm_import_module = "skrynia")]
unsafe extern "C" {
    fn invoke(req: u32, len: u32, output: u32, capacity: u32) -> i32;
}
pub struct Host;
impl Callback for Host {
    fn invoke(&self, bytes: &[u8]) -> Result<Vec<u8>, InvokeError> {
        let mut out = vec![0u8; 8 * 1024 * 1024];
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
