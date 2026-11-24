//! Replay the single pending seal intent so marker and listing agree on crash.
use super::check::is_ready;
use super::*;
use crate::index::{self, Scope};

pub fn recover<D: Disk>(disk: &D, dir: &str, iid: &[u8]) -> Result<(), StoreErr<D::Err>> {
    let base = store::ready_dir(dir, iid).ok_or(StoreErr::BadId)?;
    let scope = Scope { base: &base };
    if let Some(hash) = index::take_pending(disk, scope)? {
        if is_ready(disk, dir, iid, &hash)? {
            index::insert(disk, scope, hash)?;
        }
        index::clear_pending(disk, scope)?;
    }
    Ok(())
}
