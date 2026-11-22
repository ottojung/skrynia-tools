//! Unsubscribe with an explicit narrow per-key grant. Remove failures are
//! surfaced to the caller instead of being silently discarded.
use super::Disk;
use super::err::PushErr;
use super::grant::{Kind, verify};
use super::lock::with_lock;
use super::path::valid_key as valid_object_key;
use super::paths::sub_path;
use crate::{CAP_LEN, MutLock, hash_bytes};

pub fn unsubscribe<D: Disk, L: MutLock>(
    disk: &D,
    lock: &L,
    base: &str,
    grant_cap: &[u8],
    key: &[u8],
    subscriber: &[u8],
) -> Result<(), PushErr<D::Err, L::Err>> {
    if grant_cap.len() != CAP_LEN || !valid_object_key(key) {
        return Err(PushErr::BadAuth);
    }
    if subscriber.is_empty() || subscriber.len() > 256 {
        return Err(PushErr::BadKey);
    }
    let ns = verify(
        disk,
        base,
        &Kind::Subscribe,
        grant_cap,
        key,
        &hash_bytes(subscriber),
    )?;
    with_lock(lock, || {
        let file = sub_path(base, &ns, key, subscriber);
        if disk.read_all(&file).map_err(PushErr::Disk)?.is_none() {
            return Err(PushErr::Missing);
        }
        disk.remove(&file).map_err(PushErr::Disk)
    })
}
