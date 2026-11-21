#[cfg(target_arch = "wasm32")]
mod inner {
    #[link(wasm_import_module = "skrynia")]
    unsafe extern "C" {
        pub fn read(fd: i32, ptr: u32, len: u32) -> i32;
        pub fn write(fd: i32, ptr: u32, len: u32) -> i32;
        pub fn invoke(rp: u32, rl: u32, op: u32, ol: u32) -> i32;
    }
    pub fn read_all(fd: i32) -> alloc::vec::Vec<u8> {
        let mut out = alloc::vec::Vec::new();
        let mut buf = [0u8; 4096];
        loop {
            let n = unsafe { read(fd, buf.as_mut_ptr() as u32, buf.len() as u32) };
            if n <= 0 {
                break;
            }
            out.extend_from_slice(&buf[..n as usize]);
        }
        out
    }
    pub fn write_all(fd: i32, mut bytes: &[u8]) {
        while !bytes.is_empty() {
            let n = unsafe { write(fd, bytes.as_ptr() as u32, bytes.len() as u32) };
            if n <= 0 {
                break;
            }
            bytes = &bytes[n as usize..];
        }
    }
    pub fn invoke_raw(req: &[u8], out: &mut [u8]) -> i32 {
        unsafe {
            invoke(
                req.as_ptr() as u32,
                req.len() as u32,
                out.as_mut_ptr() as u32,
                out.len() as u32,
            )
        }
    }
}
#[cfg(target_arch = "wasm32")]
pub use inner::*;
