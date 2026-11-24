//! Copy-on-write insert/remove with journal-gated, bounded reclamation.
pub(crate) use super::{
    DIR_CAP, Disk, KEY_LEN, LEAF_CAP, Scope, StoreErr, format, mutate, serialize, state,
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
