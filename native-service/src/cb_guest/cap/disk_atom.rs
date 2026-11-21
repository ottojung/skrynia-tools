use super::disk_read::close;
use super::disk_rm::{hex, sync_dir};
use super::disk_write::{mkdir_p, write_bytes};
use super::{call, wire};
pub fn write_atomic(path: &str, bytes: &[u8]) -> Result<(), i32> {
    let (dir, _) = path.rsplit_once('/').ok_or(-1)?;
    mkdir_p(dir)?;
    let mut rr = vec![wire::RANDOM];
    wire::put_u32(&mut rr, 128);
    let (n, rnd) = call::call(&rr)?;
    if n < 0 {
        return Err(n);
    }
    let tmp = format!("{dir}/.{}.tmp", hex(&rnd));
    let mut o = vec![wire::OPEN_DISK];
    wire::put_bytes(&mut o, tmp.as_bytes());
    wire::put_u32(&mut o, 15);
    let (fd, _) = call::call(&o)?;
    if fd < 0 {
        return Err(fd);
    }
    let mut res = write_bytes(fd, bytes);
    if res.is_ok() {
        let mut s = vec![wire::SYNC];
        wire::put_u32(&mut s, fd as u32);
        match call::call(&s) {
            Ok((n, _)) if n < 0 => res = Err(n),
            Err(e) => res = Err(e),
            _ => {}
        }
    }
    let _ = close(fd);
    if let Err(e) = res {
        let mut d = vec![wire::DELETE];
        wire::put_bytes(&mut d, tmp.as_bytes());
        let _ = call::call(&d);
        return Err(e);
    }
    let mut rn = vec![wire::RENAME];
    wire::put_bytes(&mut rn, tmp.as_bytes());
    wire::put_bytes(&mut rn, path.as_bytes());
    match call::call(&rn) {
        Ok((n, _)) if n < 0 => {
            let mut d = vec![wire::DELETE];
            wire::put_bytes(&mut d, tmp.as_bytes());
            let _ = call::call(&d);
            return Err(n);
        }
        Ok(_) => sync_dir(dir),
        Err(e) => {
            // Transport/protocol failure after rename request: outcome may be
            // committed-but-ack-lost. Fail closed without claiming rollback.
            Err(e)
        }
    }
}
