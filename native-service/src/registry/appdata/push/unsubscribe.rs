use super::err::PushErr;
use super::lock::with_lock;
use super::path::valid_key as valid_object_key;
use super::paths::sub_path;
use crate::CAP_LEN;
use crate::MutLock;
use super::Disk;
use crate::read_cap;

pub fn unsubscribe<D: Disk, L: MutLock>(
    disk: &D,
    lock: &L,
    base: &str,
    write_cap: &[u8],
    key: &[u8],
    subscriber: &[u8],
) -> Result<(), PushErr<D::Err, L::Err>> {
    if write_cap.len() != CAP_LEN || !valid_object_key(key) {
        return Err(PushErr::BadAuth);
    }
    if subscriber.is_empty() || subscriber.len() > 256 {
        return Err(PushErr::BadKey);
    }
    let ns = read_cap(write_cap);
    with_lock(lock, || {
        let file = sub_path(base, &ns, key, subscriber);
        let _ = disk.remove(&file);
        Ok(())
    })
}
