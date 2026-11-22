//! Subscribe only with an explicit narrow per-key grant. A namespace write
//! capability alone is not subscribe authority: it must first issue a grant
//! bound to this exact key and this subscriber.
use super::Disk;
use super::err::PushErr;
use super::grant::{Kind, verify};
use super::id;
use super::lock::with_lock;
use super::path::valid_key as valid_object_key;
use super::paths::{sub_path, subs_dir};
use super::{MAX_SUB_BYTES, MAX_SUBS};
use crate::{CAP_LEN, MutLock, hash_bytes};

pub fn subscribe<D: Disk, L: MutLock>(
    disk: &D,
    lock: &L,
    base: &str,
    grant_cap: &[u8],
    key: &[u8],
    subscriber: &[u8],
    subscription: &[u8],
) -> Result<Vec<u8>, PushErr<D::Err, L::Err>> {
    if grant_cap.len() != CAP_LEN || !valid_object_key(key) {
        return Err(PushErr::BadAuth);
    }
    if subscriber.is_empty() || subscriber.len() > 256 {
        return Err(PushErr::BadKey);
    }
    if subscription.is_empty() || subscription.len() > MAX_SUB_BYTES {
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
