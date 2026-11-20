use super::super::abi;
pub fn allowed(path: &str) -> bool {
    if path.is_empty() || path.contains('\0') || path.contains('\\') {
        return false;
    }
    if path.starts_with('/') {
        return false;
    }
    if path
        .split('/')
        .any(|p| p.is_empty() || p == ".." || p == ".")
    {
        return false;
    }
    path == "programs" || path.starts_with("programs/")
}
pub fn hex(b: &[u8]) -> String {
    const H: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(b.len() * 2);
    for &x in b {
        s.push(H[(x >> 4) as usize] as char);
        s.push(H[(x & 0x0f) as usize] as char);
    }
    s
}
pub fn ensure(dir: &str) -> Result<(), i32> {
    let mut cur = String::new();
    for part in dir.trim_end_matches('/').split('/') {
        if part.is_empty() {
            continue;
        }
        if !cur.is_empty() {
            cur.push('/');
        }
        cur.push_str(part);
        let n = unsafe { abi::mkdir(cur.as_ptr() as u32, cur.len() as u32, 4) };
        if n == -3 {
            continue;
        }
        if n < 0 {
            return Err(n);
        }
    }
    Ok(())
}
pub fn raw_read(path: &str) -> Result<Option<Vec<u8>>, i32> {
    let fd = unsafe { abi::open_disk(path.as_ptr() as u32, path.len() as u32, 1) };
    if fd == -2 {
        return Ok(None);
    }
    if fd < 0 {
        return Err(fd);
    }
    let mut out = Vec::new();
    loop {
        let mut b = [0u8; 8192];
        let n = unsafe { abi::read(fd, b.as_mut_ptr() as u32, b.len() as u32) };
        if n <= 0 {
            unsafe { abi::close(fd) };
            if n < 0 {
                return Err(n);
            }
            return Ok(Some(out));
        }
        out.extend_from_slice(&b[..n as usize]);
    }
}
