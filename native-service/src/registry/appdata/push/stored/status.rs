//! Read-only subscriber-scoped exact subscription status in ordinary P1.
//! No endpoint material leaves P1; no durable state is written.
use super::{Disk, Kind, PushErr, by_hash, parts, sub_path, valid_key, with_lock};
use super::{Status, equal};
use crate::{CAP_LEN, MutLock, hash_bytes};

pub fn status<D: Disk, L: MutLock>(
    disk: &D,
    lock: &L,
    base: &str,
    grant_cap: &[u8],
    key: &[u8],
    subscriber: &[u8],
    expected: &[u8],
) -> Result<Status, PushErr<D::Err, L::Err>> {
    if grant_cap.len() != CAP_LEN
        || expected.len() != CAP_LEN
        || !valid_key(key)
        || subscriber.is_empty()
        || subscriber.len() > 256
    {
        return Err(PushErr::BadAuth);
    }
    with_lock(lock, || {
        let cap_hash = hash_bytes(grant_cap);
        let subject = hash_bytes(subscriber);
        let (ns, generation) = by_hash(disk, base, &Kind::Subscribe, &cap_hash, key, &subject)?;
        let file = sub_path(base, &ns, key, subscriber);
        let Some(bytes) = disk.read_all(&file).map_err(PushErr::Disk)? else {
            return Ok(Status::Absent);
        };
        let Some(p) = parts(&bytes) else {
            return Ok(Status::Invalid);
        };
        if !equal(p.cap_hash, &cap_hash)
            || !equal(p.subject_hash, &subject)
            || p.generation != generation
        {
            return Ok(Status::Invalid);
        }
        let actual = hash_bytes(p.endpoint);
        Ok(if equal(&actual, expected) {
            Status::Active(actual)
        } else {
            Status::Mismatch
        })
    })
}
