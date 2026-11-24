//! Reclaim stale endpoints under caller's shared mutation lock.
use super::subs_dir;
use super::{Disk, PushErr, active, codec};

pub fn prune<D: Disk, X>(
    disk: &D,
    base: &str,
    ns: &[u8],
    key: &[u8],
) -> Result<usize, PushErr<D::Err, X>> {
    let dir = subs_dir(base, ns, key);
    let mut live = 0;
    for name in disk.list(&dir).map_err(PushErr::Disk)? {
        if !name.ends_with(".bin") || name.contains('/') || name.as_bytes().contains(&92) {
            continue;
        }
        let path = format!("{dir}/{name}");
        let Some(bytes) = disk.read_all(&path).map_err(PushErr::Disk)? else {
            continue;
        };
        // Legacy/corrupt files are not eligible for implicit destructive cleanup.
        // They are excluded from active quotas until an explicit migration.
        if codec::parts(&bytes).is_none() {
            continue;
        }
        if active(disk, base, ns, key, &name, &bytes)?.is_some() {
            live += 1;
        } else {
            disk.remove(&path).map_err(PushErr::Disk)?;
        }
    }
    Ok(live)
}
