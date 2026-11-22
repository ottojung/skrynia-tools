//! Issue a narrow grant. Possession of the namespace write capability is the
//! only authority that may create or replace a grant record for an exact key.
use super::Disk;
use super::err::PushErr;
use super::fmt::{Kind, encode, grant_path};
use super::path;
use crate::{CAP_LEN, hash_bytes, read_cap};

pub fn issue<D: Disk, X>(
    disk: &D,
    base: &str,
    kind: &Kind,
    write_cap: &[u8],
    grant_cap: &[u8],
    key: &[u8],
    subject: &[u8],
) -> Result<(), PushErr<D::Err, X>> {
    if write_cap.len() != CAP_LEN || grant_cap.len() != CAP_LEN || !path::valid_key(key) {
        return Err(PushErr::BadAuth);
    }
    let body = encode(&read_cap(write_cap), key, subject);
    let path = grant_path(base, kind, &hash_bytes(grant_cap), key);
    disk.write_atomic(&path, &body).map_err(PushErr::Disk)
}
