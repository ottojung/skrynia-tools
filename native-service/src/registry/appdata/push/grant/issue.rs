//! Creation and replacement require namespace-writer authority under one lock.
use super::Disk;
use super::err::PushErr;
use super::fmt::{Kind, grant_path};
use super::{decode, encode, owned_by, revision};
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
        || (matches!(kind, Kind::Wake) && key != super::super::WAKE_KEY)
    {
        return Err(PushErr::BadAuth);
    }
    let ns = read_cap(write_cap);
    let path = grant_path(base, kind, &hash_bytes(grant_cap), key);
    with_lock(lock, || {
        let next = match disk.read_all(&path).map_err(PushErr::Disk)? {
            Some(old) if owned_by(&old, &ns, key) => {
                let prev = revision(&old, key).ok_or(PushErr::BadAuth)?;
                if decode(&old, key, subject).is_some() {
                    prev
                } else {
                    prev.checked_add(1).ok_or(PushErr::BadAuth)?
                }
            }
            Some(_) => return Err(PushErr::BadAuth),
            None => 1,
        };
        let body = encode(&ns, key, subject, next);
        disk.write_atomic(&path, &body).map_err(PushErr::Disk)
    })
}
