//! Verify a presented narrow grant and recover its durable namespace binding.
//! A broad write capability presented here has no matching grant record.
use super::Disk;
use super::by_hash;
use super::err::PushErr;
use super::fmt::Kind;
use super::path;
use crate::{CAP_LEN, hash_bytes};

pub fn verify<D: Disk, X>(
    disk: &D,
    base: &str,
    kind: &Kind,
    grant_cap: &[u8],
    key: &[u8],
    subject: &[u8],
) -> Result<[u8; CAP_LEN], PushErr<D::Err, X>> {
    if grant_cap.len() != CAP_LEN || !path::valid_key(key) {
        return Err(PushErr::BadAuth);
    }
    Ok(by_hash(disk, base, kind, &hash_bytes(grant_cap), key, subject)?.0)
}
