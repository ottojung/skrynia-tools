//! Exact grant subject and generation snapshot, usable only in P1.
use super::Disk;
use super::err::PushErr;
use super::fmt::{Kind, grant_path};
use super::{decode, revision};
use crate::CAP_LEN;

pub fn by_hash<D: Disk, X>(
    disk: &D,
    base: &str,
    kind: &Kind,
    cap_hash: &[u8],
    key: &[u8],
    subject: &[u8],
) -> Result<([u8; CAP_LEN], u64), PushErr<D::Err, X>> {
    if cap_hash.len() != CAP_LEN || subject.len() != CAP_LEN {
        return Err(PushErr::BadAuth);
    }
    let path = grant_path(base, kind, cap_hash, key);
    let bytes = disk
        .read_all(&path)
        .map_err(PushErr::Disk)?
        .ok_or(PushErr::BadAuth)?;
    let ns = decode(&bytes, key, subject).ok_or(PushErr::BadAuth)?;
    let generation = revision(&bytes, key).ok_or(PushErr::BadAuth)?;
    Ok((ns, generation))
}
