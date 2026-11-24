//! Replay the single pending seal intent so marker and listing agree on crash.
use super::check::is_ready;
use super::*;
use crate::index::{self, Scope};

/// A prior marker-only registry has no index meta; fail closed rather than
/// silently indexing only newly sealed markers. Clean stores initialise fine.
pub fn ensure<D: Disk>(disk: &D, dir: &str, iid: &[u8]) -> Result<(), StoreErr<D::Err>> {
    let base = store::ready_dir(dir, iid).ok_or(StoreErr::BadId)?;
    let scope = Scope { base: &base };
    if disk
        .read_all(&scope.meta())
        .map_err(StoreErr::Disk)?
        .is_some()
    {
        return Ok(());
    }
    let names = disk.list(&base).map_err(StoreErr::Disk)?;
    if names.iter().any(|n| n.ends_with(".ready")) {
        return Err(StoreErr::BadRecord);
    }
    Ok(())
}

pub fn recover<D: Disk>(disk: &D, dir: &str, iid: &[u8]) -> Result<(), StoreErr<D::Err>> {
    let base = store::ready_dir(dir, iid).ok_or(StoreErr::BadId)?;
    let scope = Scope { base: &base };
    if disk
        .read_all(&scope.meta())
        .map_err(StoreErr::Disk)?
        .is_none()
    {
        let pend = index::peek_pending(disk, scope)?;
        let allowed = pend.map(|h| format!("{}.ready", crate::id::to_hex(&h)));
        let names = disk.list(&base).map_err(StoreErr::Disk)?;
        if names
            .iter()
            .any(|n| n.ends_with(".ready") && Some(n) != allowed.as_ref())
        {
            return Err(StoreErr::BadRecord);
        }
    }
    let pend = index::take_pending(disk, scope)?;
    if let Some(hash) = pend {
        let ready = is_ready(disk, dir, iid, &hash)?;
        if ready {
            index::insert(disk, scope, hash)?;
        }
        index::clear_pending(disk, scope)?;
    }
    ensure(disk, dir, iid)?;
    Ok(())
}
