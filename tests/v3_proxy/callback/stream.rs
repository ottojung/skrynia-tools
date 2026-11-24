#[link(wasm_import_module = "skrynia")]
unsafe extern "C" {
    fn read(fd: i32, dst: u32, len: u32) -> i32;
    fn write(fd: i32, src: u32, len: u32) -> i32;
}
pub fn receive(fd: i32, buf: &mut [u8]) -> Result<usize, i32> {
    let mut len = 0;
    loop {
        if len == buf.len() {
            return Err(-1);
        }
        let tail = &mut buf[len..];
        let n = unsafe { read(fd, tail.as_mut_ptr() as u32, tail.len() as u32) };
        if n < 0 {
            return Err(n);
        }
        if n == 0 {
            return Ok(len);
        }
        len += n as usize;
    }
}
pub fn send(fd: i32, bytes: &[u8]) -> i32 {
    let n = unsafe { write(fd, bytes.as_ptr() as u32, bytes.len() as u32) };
    if n == bytes.len() as i32 { 0 } else { -1 }
}
