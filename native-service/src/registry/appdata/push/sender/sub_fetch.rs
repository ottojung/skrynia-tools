//! Bounded, sender-scoped read of stored push subscription material.
//! Endpoint/p256dh/auth bytes go ONLY to a sender holding a matching
//! (namespace, key) Kind::Sender grant — never to a public/browser caller.
use super::{Disk, MAX_SUBS, MutLock, PushErr, scope, stored, with_lock};

pub fn sub_fetch<D: Disk, L: MutLock>(
    disk: &D,
    lock: &L,
    base: &str,
    sender_cap: &[u8],
    ns: &[u8],
    key: &[u8],
) -> Result<Vec<u8>, PushErr<D::Err, L::Err>> {
    with_lock(lock, || {
        scope(disk, base, sender_cap, ns, key)?;
        let dir = super::subs_dir(base, ns, key);
        let mut blobs = Vec::new();
        for name in disk.list(&dir).map_err(PushErr::Disk)? {
            if !name.ends_with(".bin") {
                continue;
            }
            let bytes = disk
                .read_all(&format!("{dir}/{name}"))
                .map_err(PushErr::Disk)?
                .ok_or(PushErr::Missing)?;
            if let Some(endpoint) = stored::active(disk, base, ns, key, &name, &bytes)? {
                blobs.push(endpoint.to_vec());
                if blobs.len() >= MAX_SUBS {
                    break;
                }
            }
        }
        blobs.sort();
        let mut out = (blobs.len() as u32).to_le_bytes().to_vec();
        for blob in &blobs {
            out.extend_from_slice(&(blob.len() as u32).to_le_bytes());
            out.extend_from_slice(blob);
        }
        Ok(out)
    })
}
