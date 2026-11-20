use super::{dir, file, io, path, rand};
use crate::bridge::{Callback, InvokeError};

pub fn mkdir_p<I: Callback>(i: &I, name: &str) -> Result<(), InvokeError> {
    if name.is_empty() { return Ok(()); }
    let mut cur = String::new();
    for part in name.split('/').filter(|p| !p.is_empty()) {
        if !cur.is_empty() { cur.push('/'); }
        cur.push_str(part);
        let rc = path::mkdir(i, &cur, 4)?;
        if rc < 0 && path::exists(i, &cur)? != 1 { return Err(InvokeError::Unavailable); }
    }
    Ok(())
}
pub fn write_atomic<I: Callback>(i: &I, name: &str, bytes: &[u8]) -> Result<(), InvokeError> {
    if let Some((parent, _)) = name.rsplit_once('/') { mkdir_p(i, parent)?; }
    let rnd = rand::bytes(i, 8)?;
    let tmp = format!("{name}.tmp.{:016x}", u64::from_le_bytes(rnd.try_into().unwrap()));
    let fd = file::open_disk(i, &tmp, 2 + 4 + 8 + 16)?; // write|create|exclusive|truncate
    let res = (|| { io::write_all_fd(i, fd, bytes)?; if path::sync(i, fd)? < 0 { Err(InvokeError::Unavailable) } else { Ok(()) } })();
    let _ = file::close(i, fd);
    res?;
    if path::rename(i, &tmp, name)? < 0 { let _ = path::delete(i, &tmp); return Err(InvokeError::Unavailable); }
    if let Some((parent, _)) = name.rsplit_once('/') { sync_dir(i, parent)?; }
    Ok(())
}
fn sync_dir<I: Callback>(i: &I, name: &str) -> Result<(), InvokeError> {
    let fd = dir::open(i, name)?;
    let rc = path::sync(i, fd)?;
    let _ = file::close(i, fd);
    if rc < 0 { Err(InvokeError::Unavailable) } else { Ok(()) }
}
pub fn list_dir<I: Callback>(i: &I, name: &str) -> Result<Vec<String>, InvokeError> {
    let fd = dir::open(i, name)?;
    let mut out = Vec::new();
    loop {
        let (st, data) = dir::next(i, fd)?;
        if st == 0 { break; }
        if st < 0 { let _ = file::close(i, fd); return Err(InvokeError::Unavailable); }
        out.push(String::from_utf8(data).map_err(|_| InvokeError::Unavailable)?);
    }
    let _ = file::close(i, fd);
    Ok(out)
}
