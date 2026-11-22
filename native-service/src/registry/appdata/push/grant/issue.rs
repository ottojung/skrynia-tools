//! Creation and replacement require namespace-writer authority under one lock.
use super::Disk;
use super::err::PushErr;
use super::fmt::{Kind, encode, grant_path, owned_by};
use super::{path, with_lock};
use crate::{CAP_LEN, MutLock, hash_bytes, read_cap};

#[allow(clippy::too_many_arguments)]
pub fn issue<D: Disk, L: MutLock>(
    disk: &D,
    lock: &L,
    base: &str,
    kind: &Kind,
    write_cap: &[u8],
    grant_cap: &[u8],
    key: &[u8],
    subject: &[u8],
) -> Result<(), PushErr<D::Err, L::Err>> {
    if write_cap.len() != CAP_LEN
        || grant_cap.len() != CAP_LEN
        || subject.len() != CAP_LEN
        || !path::valid_key(key)
    {
        return Err(PushErr::BadAuth);
    }
    let ns = read_cap(write_cap);
    let body = encode(&ns, key, subject);
    let path = grant_path(base, kind, &hash_bytes(grant_cap), key);
    with_lock(lock, || {
        if let Some(old) = disk.read_all(&path).map_err(PushErr::Disk)?
            && !owned_by(&old, &ns, key)
        {
            return Err(PushErr::BadAuth);
        }
        disk.write_atomic(&path, &body).map_err(PushErr::Disk)
    })
}
