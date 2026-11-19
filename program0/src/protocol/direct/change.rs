use super::{Disk, err, ok, store_io};
pub fn set<D: Disk>(disk: &D, id: &[u8], code: &[u8]) -> Vec<u8> {
    if store_io::get(disk, "programs", id).is_err() {
        return err(3);
    }
    match store_io::put(disk, "programs", id, code) {
        Ok(()) => ok(Vec::new()),
        Err(_) => err(3),
    }
}
pub fn revoke<D: Disk>(disk: &D, id: &[u8]) -> Vec<u8> {
    if store_io::get(disk, "programs", id).is_err() {
        return err(6);
    }
    match store_io::del(disk, "programs", id) {
        Ok(()) => ok(Vec::new()),
        Err(_) => err(6),
    }
}
