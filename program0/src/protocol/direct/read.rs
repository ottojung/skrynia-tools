use super::{Disk, err, ok, store_io};
pub fn lookup<D: Disk>(disk: &D, id: &[u8]) -> Vec<u8> {
    match store_io::get(disk, "programs", id) {
        Ok(code) => ok(code),
        Err(_) => err(5),
    }
}
