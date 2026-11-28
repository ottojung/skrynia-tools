use super::{call, wire};

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
