//! Stage local and cross-namespace hints before an authorized source commit.
use super::{Disk, push};

pub fn stage<D: Disk>(
    disk: &D,
    base: &str,
    ns: &[u8],
    key: &[u8],
    hash: Option<[u8; 32]>,
    wake_ns: &[u8],
) -> Result<(), push::PushErr<D::Err, ()>> {
    if key == push::WAKE_KEY {
        push::mark_wake_dirty(disk, base, ns, key)?;
    } else {
        push::enqueue_hints(disk, base, ns, key, hash)?;
        push::mark_wake_dirty(disk, base, ns, push::WAKE_KEY)?;
    }
    if wake_ns != ns {
        push::mark_wake_dirty(disk, base, wake_ns, push::WAKE_KEY)?;
    }
    Ok(())
}
