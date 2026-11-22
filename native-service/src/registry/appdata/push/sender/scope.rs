//! Sender identity check. A sender scope grant binds one exact (ns, key) pair;
//! the sender capability is distinct from any write grant or root token.
use super::{CAP_LEN, Disk, Kind, PushErr, verify_grant};

pub(crate) fn scope<D: Disk, X>(
    disk: &D,
    base: &str,
    sender_cap: &[u8],
    ns: &[u8],
    key: &[u8],
) -> Result<(), PushErr<D::Err, X>> {
    if sender_cap.len() != CAP_LEN || ns.len() != CAP_LEN {
        return Err(PushErr::BadAuth);
    }
    let granted = verify_grant(disk, base, &Kind::Sender, sender_cap, key, &[0u8; CAP_LEN])?;
    if granted.as_slice() != ns {
        return Err(PushErr::BadAuth);
    }
    Ok(())
}
