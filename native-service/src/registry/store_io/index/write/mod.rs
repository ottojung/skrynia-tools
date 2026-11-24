//! Copy-on-write insert/remove; meta is the commit point, retired nodes reclaimed.
pub(crate) use super::{
    DIR_CAP, Disk, KEY_LEN, LEAF_CAP, Scope, StoreErr, format, mutate, serialize, state,
};
mod change;

pub fn insert<D: Disk>(disk: &D, s: Scope, key: [u8; KEY_LEN]) -> Result<(), StoreErr<D::Err>> {
    change::change(disk, s, key, true)
}

pub fn remove<D: Disk>(disk: &D, s: Scope, key: [u8; KEY_LEN]) -> Result<(), StoreErr<D::Err>> {
    change::change(disk, s, key, false)
}
