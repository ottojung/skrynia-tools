use super::disk_read::close;
use super::{call, wire};
pub fn list(dir: &str) -> Result<Vec<String>, i32> {
    let mut req = vec![wire::OPEN_DIR];
    wire::put_bytes(&mut req, dir.as_bytes());
    let (fd, _) = call::call(&req)?;
    if wire::is_missing(fd) {
        return Ok(Vec::new());
    }
    if fd < 0 {
        return Err(fd);
    }
    let mut names = Vec::new();
    loop {
        let mut r = vec![wire::NEXT_ENTRY];
        wire::put_u32(&mut r, fd as u32);
        wire::put_u32(&mut r, 256);
        let (n, b) = call::call(&r)?;
        if n <= 0 {
            let _ = close(fd);
            if n < 0 {
                return Err(n);
            }
            return Ok(names);
        }
        names.push(String::from_utf8(b[..n as usize].to_vec()).map_err(|_| -1)?);
    }
}
