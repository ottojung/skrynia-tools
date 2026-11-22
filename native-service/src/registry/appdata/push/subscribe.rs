//! Subscribe only with an explicit narrow per-key grant. A namespace write
//! capability alone is not subscribe authority: it must first issue a grant
//! bound to this exact key and this subscriber.
use super::Disk;
use super::err::PushErr;
use super::grant::{Kind, by_hash};
use super::lock::with_lock;
use super::path::valid_key as valid_object_key;
use super::paths::sub_path;
use super::stored;
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
    with_lock(lock, || {
        let grant_hash = hash_bytes(grant_cap);
        let subject = hash_bytes(subscriber);
        let (ns, generation) = by_hash(disk, base, &Kind::Subscribe, &grant_hash, key, &subject)?;
        let file = sub_path(base, &ns, key, subscriber);
        let exists = disk.read_all(&file).map_err(PushErr::Disk)?.is_some();
        let count = stored::prune(disk, base, &ns, key)?;
        if !exists && count >= MAX_SUBS {
            return Err(PushErr::Quota);
        }
        let body = stored::encode(&grant_hash, generation, &subject, subscription);
        disk.write_atomic(&file, &body).map_err(PushErr::Disk)?;
        Ok(Vec::new())
    })
}
