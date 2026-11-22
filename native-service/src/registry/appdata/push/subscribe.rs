use super::Disk;
use super::err::PushErr;
use super::id;
use super::lock::with_lock;
use super::path::valid_key as valid_object_key;
use super::paths::{sub_path, subs_dir};
use super::{MAX_SUB_BYTES, MAX_SUBS};
use crate::CAP_LEN;
use crate::MutLock;
use crate::read_cap;

pub fn subscribe<D: Disk, L: MutLock>(
    disk: &D,
    lock: &L,
    base: &str,
    write_cap: &[u8],
    key: &[u8],
    subscriber: &[u8],
    subscription: &[u8],
) -> Result<Vec<u8>, PushErr<D::Err, L::Err>> {
    if write_cap.len() != CAP_LEN || !valid_object_key(key) {
        return Err(PushErr::BadAuth);
    }
    if subscriber.is_empty() || subscriber.len() > 256 {
        return Err(PushErr::BadKey);
    }
    if subscription.is_empty() || subscription.len() > MAX_SUB_BYTES {
        return Err(PushErr::BadKey);
    }
    let ns = read_cap(write_cap);
    with_lock(lock, || {
        let dir = subs_dir(base, &ns, key);
        let file = sub_path(base, &ns, key, subscriber);
        let exists = disk.read_all(&file).map_err(PushErr::Disk)?.is_some();
        let count = disk.list(&dir).map_err(PushErr::Disk)?.len();
        if !exists && count >= MAX_SUBS {
            return Err(PushErr::Quota);
        }
        disk.write_atomic(&file, subscription)
            .map_err(PushErr::Disk)?;
        let _ = id::validate(&ns);
        Ok(Vec::new())
    })
}
