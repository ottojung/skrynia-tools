//! Bounded, ordered hint sequence listing for a scoped sender. The reserved
//! wake key publishes only its current dirty generation, never hint files.
use super::{
    Disk, MAX_HINTS, MutLock, PushErr, WAKE_KEY, hints_root, parse_seq, scope, wake_current,
    with_lock,
};

pub fn hint_list<D: Disk, L: MutLock>(
    disk: &D,
    lock: &L,
    base: &str,
    sender_cap: &[u8],
    ns: &[u8],
    key: &[u8],
) -> Result<Vec<u8>, PushErr<D::Err, L::Err>> {
    with_lock(lock, || {
        scope(disk, base, sender_cap, ns, key)?;
        let mut seqs = Vec::new();
        if key == WAKE_KEY {
            let (generation, dirty) = wake_current(disk, base, ns, key)?;
            if dirty {
                seqs.push(generation);
            }
        } else {
            for name in disk
                .list(&hints_root(base, ns, key))
                .map_err(PushErr::Disk)?
            {
                if let Some(seq) = parse_seq(&name) {
                    seqs.push(seq);
                }
            }
        }
        seqs.sort_unstable();
        seqs.truncate(MAX_HINTS);
        let mut out = (seqs.len() as u32).to_le_bytes().to_vec();
        for seq in seqs {
            out.extend_from_slice(&seq.to_le_bytes());
        }
        Ok(out)
    })
}
