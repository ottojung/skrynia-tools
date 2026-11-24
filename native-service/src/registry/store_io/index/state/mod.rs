//! Load the three-node search path as mutable working entries.
mod finish;
mod load;
pub(super) mod reclaim;
mod util;

use super::pending::scope;
use super::{
    DIR_CAP, Disk, KEY_LEN, LEAF_CAP, ROOT_CAP, Scope, StoreErr, find, format, mutate, read,
};

pub struct State {
    pub nd: u32,
    pub nl: u32,
    pub generation: u32,
    pub root: Vec<mutate::DirEntry>,
    pub di: usize,
    pub dir: Vec<mutate::DirEntry>,
    pub leaf: Vec<mutate::LeafEntry>,
    pub old_root: u32,
    pub old_dir: u32,
    pub old_leaf: u32,
}

pub use load::load;

/// Reject an unsupported pre-index store instead of silently reindexing only
/// new records. A clean, empty store may initialise normally.
pub fn ensure_program<D: Disk>(disk: &D, dir: &str) -> Result<(), StoreErr<D::Err>> {
    let s = scope(dir);
    if read(disk, s.meta())?.is_some() {
        return Ok(());
    }
    let active = format!("{}/active", dir.trim_end_matches('/'));
    let names = disk.list(&active).map_err(StoreErr::Disk)?;
    if names.iter().any(|n| n.ends_with(".bin")) {
        return Err(StoreErr::BadRecord);
    }
    Ok(())
}

/// Validate meta and capacities before any caller writes a pending intent or
/// a durable marker, so capacity failures cannot strand unrecoverable state.
pub fn preflight<D: Disk>(disk: &D, s: Scope) -> Result<(), StoreErr<D::Err>> {
    let Some(raw) = read(disk, s.meta())? else {
        return Ok(());
    };
    let body = format::open(&raw).ok_or(StoreErr::BadRecord)?;
    let (nd, nl, g) = format::parse_meta(&body).ok_or(StoreErr::BadRecord)?;
    if nd == u32::MAX || nl == u32::MAX || g == u32::MAX {
        return Err(StoreErr::BadRecord);
    }
    Ok(())
}
