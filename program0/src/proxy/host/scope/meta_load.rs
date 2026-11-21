use super::{abi, meta_dec};

pub fn load(program: &[u8]) -> Option<super::meta::Policy> {
    let mpath = meta_dec::meta_path(program)?;
    let fd = unsafe { abi::open_disk(mpath.as_ptr() as u32, mpath.len() as u32, 1) };
    if fd < 0 {
        return None;
    }
    let mut buf = [0u8; 256];
    let mut n = 0usize;
    while n < buf.len() {
        let r = unsafe { abi::read(fd, buf[n..].as_mut_ptr() as u32, (buf.len() - n) as u32) };
        if r <= 0 {
            break;
        }
        n += r as usize;
    }
    unsafe { abi::close(fd) };
    meta_dec::parse(&buf[..n])
}
