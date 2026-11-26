use super::{call, wire};
pub fn read_all(path: &str) -> Result<Option<Vec<u8>>, i32> {
    let mut req = vec![wire::OPEN_DISK];
    wire::put_bytes(&mut req, path.as_bytes());
    wire::put_u32(&mut req, 1);
    let (fd, _) = call::call(&req)?;
    if wire::is_missing(fd) {
        return Ok(None);
    }
    if fd < 0 {
        return Err(fd);
    }
    let mut out = Vec::new();
    loop {
        let mut r = vec![wire::READ];
        wire::put_u32(&mut r, fd as u32);
        wire::put_u32(&mut r, 8192);
        let read = call::call_with_capacity(&r, 8192 + 4);
        let (n, b) = match read {
            Ok(v) => v,
            Err(e) => {
                let _ = close(fd);
                return Err(e);
            }
        };
        if n <= 0 {
            let _ = close(fd);
            if n < 0 {
                return Err(n);
            }
            return Ok(Some(out));
        }
        out.extend_from_slice(&b[..n as usize]);
    }
}
pub fn close(fd: i32) -> Result<(), i32> {
    let mut r = vec![wire::CLOSE];
    wire::put_u32(&mut r, fd as u32);
    let (n, _) = call::call(&r)?;
    if n < 0 { Err(n) } else { Ok(()) }
}
