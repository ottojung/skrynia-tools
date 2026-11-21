use super::disk_read::close;
use super::{call, wire};
pub fn hex(b: &[u8]) -> String {
    const H: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(b.len() * 2);
    for &x in b {
        s.push(H[(x >> 4) as usize] as char);
        s.push(H[(x & 0x0f) as usize] as char);
    }
    s
}
pub fn remove(path: &str) -> Result<(), i32> {
    let mut r = vec![wire::DELETE];
    wire::put_bytes(&mut r, path.as_bytes());
    let (n, _) = call::call(&r)?;
    if n < 0 {
        return Err(n);
    }
    if let Some((dir, _)) = path.rsplit_once('/') {
        let _ = sync_dir(dir);
    }
    Ok(())
}
pub fn sync_dir(dir: &str) -> Result<(), i32> {
    let mut req = vec![wire::OPEN_DIR];
    wire::put_bytes(&mut req, dir.as_bytes());
    let (fd, _) = call::call(&req)?;
    if fd < 0 {
        return Err(fd);
    }
    let mut s = vec![wire::SYNC];
    wire::put_u32(&mut s, fd as u32);
    let (n, _) = call::call(&s)?;
    let _ = close(fd);
    if n < 0 { Err(n) } else { Ok(()) }
}
