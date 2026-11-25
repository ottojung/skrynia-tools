//! Copy-on-write insert/remove with journal-gated, bounded reclamation.
pub(crate) use super::{
    DIR_CAP, Disk, KEY_LEN, LEAF_CAP, Scope, StoreErr, format, serialize, state,
};
mod change;
mod plan;
mod plan_dir;
mod plan_leaf;
mod plan_nodes;
mod plan_writes;

pub fn insert<D: Disk>(disk: &D, s: Scope, key: [u8; KEY_LEN]) -> Result<(), StoreErr<D::Err>> {
    change::change(disk, s, key, true)
}

pub fn remove<D: Disk>(disk: &D, s: Scope, key: [u8; KEY_LEN]) -> Result<(), StoreErr<D::Err>> {
    change::change(disk, s, key, false)
}

/// Capacity/overflow preflight without durable mutation beyond journal replay.
pub fn dry_run<D: Disk>(
    disk: &D,
    s: Scope,
    key: [u8; KEY_LEN],
    insert: bool,
) -> Result<(), StoreErr<D::Err>> {
    let Some(mut st) = state::load(disk, s, key, insert)? else {
        return Ok(());
    };
    let _ = plan::allocate::<D>(&mut st)?;
    Ok(())
}
