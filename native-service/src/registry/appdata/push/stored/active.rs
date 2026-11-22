//! Only still-authorized exact-key subscriptions reach the sender.
use super::{Disk, Kind, PushErr, by_hash, codec, id};

pub fn active<'a, D: Disk, X>(
    disk: &D,
    base: &str,
    ns: &[u8],
    key: &[u8],
    filename: &str,
    bytes: &'a [u8],
) -> Result<Option<&'a [u8]>, PushErr<D::Err, X>> {
    let Some(codec::Parts {
        cap_hash,
        generation,
        subject_hash,
        endpoint,
    }) = codec::parts(bytes)
    else {
        return Ok(None); // Legacy or corrupt records never deliver.
    };
    if filename != format!("{}.bin", id::to_hex(subject_hash)) {
        return Ok(None);
    }
    match by_hash::<D, X>(disk, base, &Kind::Subscribe, cap_hash, key, subject_hash) {
        Ok((granted, current)) if granted == ns && current == generation => Ok(Some(endpoint)),
        Err(PushErr::Disk(e)) => Err(PushErr::Disk(e)),
        _ => Ok(None),
    }
}
