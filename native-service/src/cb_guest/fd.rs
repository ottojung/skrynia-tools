use super::cap::{call, wire};
const MAX_REQ: usize = 16 * 1024 * 1024;

pub fn read_all(fd: i32) -> Result<Vec<u8>, i32> {
    let mut out = Vec::new();
    loop {
        let mut req = vec![wire::READ];
        wire::put_u32(&mut req, fd as u32);
        wire::put_u32(&mut req, 8192);
        let (n, b) = call::call(&req)?;
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
        out.extend_from_slice(&b[..n]);
    }
}
pub fn write_all(fd: i32, data: &[u8]) -> Result<(), i32> {
    for ch in data.chunks(8192) {
        let mut done = 0;
        while done < ch.len() {
            let mut req = vec![wire::WRITE];
            wire::put_u32(&mut req, fd as u32);
            wire::put_bytes(&mut req, &ch[done..]);
            let (n, _) = call::call(&req)?;
            if n <= 0 {
                return Err(if n == 0 { -5 } else { n });
            }
            done += n as usize;
        }
    }
    Ok(())
}
