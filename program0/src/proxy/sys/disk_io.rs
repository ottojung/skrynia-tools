use super::super::abi;
use super::disk::{ensure, hex};
use super::proto;
pub fn raw_list(dir: &str) -> Result<Vec<String>, i32> {
    let fd = unsafe { abi::open_directory(dir.as_ptr() as u32, dir.len() as u32) };
    if fd == -2 {
        return Ok(Vec::new());
    }
    if fd < 0 {
        return Err(fd);
    }
    let mut names = Vec::new();
    loop {
        let mut b = [0u8; 256];
        let n = unsafe { abi::next_entry(fd, b.as_mut_ptr() as u32, b.len() as u32) };
        if n <= 0 {
            unsafe { abi::close(fd) };
            if n < 0 {
                return Err(n);
            }
            return Ok(names);
        }
        names.push(String::from_utf8(b[..n as usize].to_vec()).map_err(|_| -1)?);
    }
}
fn write_bytes(fd: i32, bytes: &[u8]) -> Result<(), i32> {
    for b in bytes.chunks(8192) {
        let mut done = 0;
        while done < b.len() {
            let n = unsafe { abi::write(fd, b[done..].as_ptr() as u32, (b.len() - done) as u32) };
            if n <= 0 {
                return Err(if n == 0 { -5 } else { n });
            }
            done += n as usize;
        }
    }
    let rc = unsafe { abi::sync(fd) };
    if rc < 0 { Err(rc) } else { Ok(()) }
}
pub fn raw_write(path: &str, bytes: &[u8]) -> Result<(), i32> {
    let (dir, _) = path.rsplit_once('/').ok_or(-1)?;
    ensure(dir)?;
    let mut rnd = [0u8; 16];
    let rc = unsafe { abi::secure_random_bits(rnd.as_mut_ptr() as u32, 128) };
    if rc < 0 {
        return Err(rc);
    }
    let tmp = format!("{dir}/.{}.tmp", hex(&rnd));
    let fd = unsafe { abi::open_disk(tmp.as_ptr() as u32, tmp.len() as u32, 15) };
    if fd < 0 {
        return Err(fd);
    }
    let mut res = write_bytes(fd, bytes);
    if res.is_ok() {
        let s = unsafe { abi::sync(fd) };
        if s < 0 {
            res = Err(s);
        }
    }
    let c = unsafe { abi::close(fd) };
    if res.is_err() || c < 0 {
        let _ = unsafe { abi::delete(tmp.as_ptr() as u32, tmp.len() as u32) };
        return Err(res.err().unwrap_or(c));
    }
    let n = unsafe {
        abi::rename(
            tmp.as_ptr() as u32,
            tmp.len() as u32,
            path.as_ptr() as u32,
            path.len() as u32,
        )
    };
    if n < 0 {
        let _ = unsafe { abi::delete(tmp.as_ptr() as u32, tmp.len() as u32) };
        return Err(n);
    }
    let dfd = unsafe { abi::open_directory(dir.as_ptr() as u32, dir.len() as u32) };
    if dfd >= 0 {
        let _ = unsafe { abi::sync(dfd) };
        unsafe { abi::close(dfd) };
    }
    Ok(())
}
pub fn raw_remove(path: &str) -> Result<(), i32> {
    let n = unsafe { abi::delete(path.as_ptr() as u32, path.len() as u32) };
    if n < 0 {
        return Err(n);
    }
    if let Some((dir, _)) = path.rsplit_once('/') {
        let dfd = unsafe { abi::open_directory(dir.as_ptr() as u32, dir.len() as u32) };
        if dfd >= 0 {
            let _ = unsafe { abi::sync(dfd) };
            unsafe { abi::close(dfd) };
        }
    }
    Ok(())
}
