//! Idempotent node reclamation.
// Absent paths are tolerated; a real delete/sync error propagates so the
// journal stays until a retry completes. Recovery is idempotent.
pub fn drop_node<D: super::super::Disk>(
    disk: &D,
    s: super::super::Scope,
    kind: u8,
    id: u32,
) -> Result<(), super::super::StoreErr<D::Err>> {
    if id == 0 {
        return Ok(());
    }
    let p = match kind {
        b'l' => s.leaf(id),
        b'd' => s.dir(id),
        _ => s.root(id),
    };
    if super::super::read(disk, p.clone())?.is_none() {
        return Ok(());
    }
    disk.remove(&p).map_err(super::super::StoreErr::Disk)
}
