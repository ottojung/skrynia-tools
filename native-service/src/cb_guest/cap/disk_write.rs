use super::{call, wire};
pub fn mkdir_p(dir: &str) -> Result<(), i32> {
    let mut cur = String::new();
    for part in dir.trim_end_matches('/').split('/') {
        if part.is_empty() {
            continue;
        }
        if !cur.is_empty() {
            cur.push('/');
        }
        cur.push_str(part);
        let mut r = vec![wire::MKDIR];
        wire::put_bytes(&mut r, cur.as_bytes());
        wire::put_u32(&mut r, 4);
        let (n, _) = call::call(&r)?;
        if n == -3 {
            continue;
        }
        if n < 0 {
            return Err(n);
        }
    }
    Ok(())
}
pub fn write_bytes(fd: i32, bytes: &[u8]) -> Result<(), i32> {
    for ch in bytes.chunks(8192) {
        let mut done = 0;
        while done < ch.len() {
            let mut r = vec![wire::WRITE];
            wire::put_u32(&mut r, fd as u32);
            wire::put_bytes(&mut r, &ch[done..]);
            let (n, _) = call::call(&r)?;
            if n <= 0 {
                return Err(if n == 0 { -5 } else { n });
            }
            done += n as usize;
        }
    }
    let mut s = vec![wire::SYNC];
    wire::put_u32(&mut s, fd as u32);
    let (n, _) = call::call(&s)?;
    if n < 0 { Err(n) } else { Ok(()) }
}
