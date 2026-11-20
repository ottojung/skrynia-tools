use super::*;
pub(super) fn marker(hash: &[u8; 32]) -> Vec<u8> {
    let mut out = vec![1];
    out.extend_from_slice(hash);
    out
}
pub fn is_ready<D: Disk>(
    disk: &D,
    dir: &str,
    iid: &[u8],
    hash: &[u8; 32],
) -> Result<bool, StoreErr<D::Err>> {
    let p = store::ready_path(dir, iid, hash).ok_or(StoreErr::BadId)?;
    match disk.read_all(&p).map_err(StoreErr::Disk)? {
        None => Ok(false),
        Some(b) if b == marker(hash) => Ok(true),
        Some(_) => Err(StoreErr::BadRecord),
    }
}
