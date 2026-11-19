use super::{Disk, Random, err, ok, store_io};
pub fn register<D: Disk, R: Random>(disk: &D, rng: &R, code: &[u8]) -> Vec<u8> {
    let mut id = vec![0; crate::id::ID_LEN];
    if rng.fill(&mut id).is_err() {
        return err(2);
    }
    match store_io::get(disk, "programs", &id) {
        Err(store_io::StoreErr::Missing) => {}
        _ => return err(2),
    }
    match store_io::put(disk, "programs", &id, code) {
        Ok(()) => ok(id),
        Err(_) => err(2),
    }
}
