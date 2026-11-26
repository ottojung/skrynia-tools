use super::disk_read::close;
use super::disk_rm::rename_sync;
use super::disk_write::{mkdir_p, write_bytes};
use super::{call, wire};
fn hex(b: &[u8]) -> String {
    const H: &[u8; 16] = b"0123456789abcdef";
    b.iter()
        .flat_map(|&x| [H[(x >> 4) as usize], H[(x & 15) as usize]])
        .map(char::from)
        .collect()
}
pub fn write_atomic(path: &str, bytes: &[u8]) -> Result<(), i32> {
    let (dir, _) = path.rsplit_once('/').ok_or(-1)?;
    mkdir_p(dir)?;
    let mut rr = vec![wire::RANDOM];
    wire::put_u32(&mut rr, 128);
    let (n, rnd) = call::call(&rr)?;
    // RANDOM replies with the exact byte count: 128 bits must be 16 bytes. A
    // short or malformed reply would yield a predictable temporary name.
    if n != 16 || rnd.len() != 16 {
        return Err(if n < 0 { n } else { -1 });
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
        // write_bytes already synced the file; this second sync is retained
        // so a genuine sync failure during the atomic commit still surfaces.
        let mut s = vec![wire::SYNC];
        wire::put_u32(&mut s, fd as u32);
        match call::call(&s) {
            Ok((n, _)) if n < 0 => res = Err(n),
            Err(e) => res = Err(e),
            _ => {}
        }
    }
    // A failed close is a real durability failure, never silently ignored.
    res = res.and(close(fd));
    if let Err(e) = res {
        let mut d = vec![wire::DELETE];
        wire::put_bytes(&mut d, tmp.as_bytes());
        let _ = call::call(&d);
        return Err(e);
    }
    rename_sync(&tmp, path, dir)
}
