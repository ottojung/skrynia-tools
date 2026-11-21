const MAX_REQ: usize = 16 * 1024 * 1024;

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "skrynia")]
unsafe extern "C" {
    fn read(fd: i32, dst: u32, len: u32) -> i32;
    fn write(fd: i32, src: u32, len: u32) -> i32;
}

pub fn read_all(fd: i32) -> Result<Vec<u8>, i32> {
    let mut out = Vec::new();
    #[cfg(not(target_arch = "wasm32"))]
    let _ = fd;
    #[cfg(target_arch = "wasm32")]
    let mut buf = [0u8; 8192];
    loop {
        #[cfg(target_arch = "wasm32")]
        let n = unsafe { read(fd, buf.as_mut_ptr() as u32, buf.len() as u32) };
        #[cfg(not(target_arch = "wasm32"))]
        let n = 0;
        if n < 0 {
            return Err(n);
        }
        if n == 0 {
            return Ok(out);
        }
        let n = n as usize;
        if out.len().saturating_add(n) > MAX_REQ {
            return Err(-11);
        }
        #[cfg(target_arch = "wasm32")]
        out.extend_from_slice(&buf[..n]);
    }
}

pub fn write_all(fd: i32, data: &[u8]) -> Result<(), i32> {
    #[cfg(not(target_arch = "wasm32"))]
    let _ = (fd, data);
    #[cfg(target_arch = "wasm32")]
    let mut off = 0;
    #[cfg(target_arch = "wasm32")]
    while off < data.len() {
        let end = (off + 8192).min(data.len());
        let chunk = &data[off..end];
        let n = unsafe { write(fd, chunk.as_ptr() as u32, chunk.len() as u32) };
        if n <= 0 {
            return Err(if n == 0 { -5 } else { n });
        }
        off += n as usize;
    }
    Ok(())
}
