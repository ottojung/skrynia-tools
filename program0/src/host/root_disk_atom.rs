#[cfg(target_arch = "wasm32")]
use super::{id, root_abi as host, root_disk_io, root_disk_ls};

#[cfg(target_arch = "wasm32")]
pub fn write_atomic(path: &str, bytes: &[u8]) -> Result<(), i32> {
    let (dir, _) = path.rsplit_once('/').ok_or(-1)?;
    root_disk_ls::ensure(dir)?;
    let mut rnd = [0u8; 16];
    let rc = unsafe { host::secure_random_bits(rnd.as_mut_ptr() as u32, 128) };
    if rc < 0 {
        return Err(rc);
    }
    let tmp = format!("{dir}/.{}.tmp", id::to_hex(&rnd));
    let fd = unsafe { host::open_disk(tmp.as_ptr() as u32, tmp.len() as u32, 15) };
    if fd < 0 {
        return Err(fd);
    }
    let mut res = root_disk_io::write_bytes(fd, bytes);
    if res.is_ok() {
        let s = unsafe { host::sync(fd) };
        if s < 0 {
            res = Err(s);
        }
    }
    let c = unsafe { host::close(fd) };
    if res.is_err() || c < 0 {
        let _ = unsafe { host::delete(tmp.as_ptr() as u32, tmp.len() as u32) };
        return Err(res.err().unwrap_or(c));
    }
    let n = unsafe {
        host::rename(
            tmp.as_ptr() as u32,
            tmp.len() as u32,
            path.as_ptr() as u32,
            path.len() as u32,
        )
    };
    if n < 0 {
        let _ = unsafe { host::delete(tmp.as_ptr() as u32, tmp.len() as u32) };
        return Err(n);
    }
    let dfd = unsafe { host::open_directory(dir.as_ptr() as u32, dir.len() as u32) };
    if dfd < 0 {
        return Err(dfd);
    }
    let ds = unsafe { host::sync(dfd) };
    let _ = unsafe { host::close(dfd) };
    if ds < 0 { Err(ds) } else { Ok(()) }
}
