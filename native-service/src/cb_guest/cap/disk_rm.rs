use super::disk_read::close;
use super::{call, wire};
pub fn remove(path: &str) -> Result<(), i32> {
    let mut r = vec![wire::DELETE];
    wire::put_bytes(&mut r, path.as_bytes());
    let (n, _) = call::call(&r)?;
    if n < 0 {
        return Err(n);
    }
    // Directory durability is part of remove: a failed parent sync must not
    // be reported as a completed deletion.
    if let Some((dir, _)) = path.rsplit_once('/') {
        sync_dir(dir)?;
    }
    Ok(())
}
pub fn sync_dir(dir: &str) -> Result<(), i32> {
    let mut req = vec![wire::OPEN_DIR];
    wire::put_bytes(&mut req, dir.as_bytes());
    let (fd, _) = call::call(&req)?;
    // ENOENT here is a genuine missing directory, not absence that may be
    // ignored: the caller needs that directory to be durable.
    if fd < 0 {
        return Err(fd);
    }
    let mut s = vec![wire::SYNC];
    wire::put_u32(&mut s, fd as u32);
    let sync = call::call(&s);
    let close_res = close(fd);
    let (n, _) = sync?;
    if n < 0 {
        return Err(n);
    }
    close_res
}
pub fn rename_sync(tmp: &str, path: &str, dir: &str) -> Result<(), i32> {
    let mut rn = vec![wire::RENAME];
    wire::put_bytes(&mut rn, tmp.as_bytes());
    wire::put_bytes(&mut rn, path.as_bytes());
    match call::call(&rn) {
        Ok((n, _)) if n < 0 => {
            let mut d = vec![wire::DELETE];
            wire::put_bytes(&mut d, tmp.as_bytes());
            let _ = call::call(&d);
            Err(n)
        }
        Ok(_) => sync_dir(dir),
        Err(e) => Err(e),
    }
}
