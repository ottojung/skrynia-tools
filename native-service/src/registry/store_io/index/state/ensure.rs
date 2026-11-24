//! Explicit legacy-store detection; clean initialisation stays write-free.
use super::scope;
use super::{Disk, KEY_LEN, Scope, StoreErr, format, read};

/// Reject an unsupported pre-index store instead of silently reindexing only
/// new records. A clean, empty store may initialise normally.
pub fn ensure_program<D: Disk>(disk: &D, dir: &str) -> Result<(), StoreErr<D::Err>> {
    let s = scope(dir);
    if read(disk, s.meta())?.is_some() {
        return Ok(());
    }
    let allowed = read(disk, s.pending())?
        .and_then(|raw| format::open(&raw))
        .and_then(|b| {
            if b.len() == KEY_LEN + 1 {
                Some(b[..KEY_LEN].to_vec())
            } else {
                None
            }
        });
    let allowed = allowed.map(|id| format!("{}.bin", crate::id::to_hex(&id)));
    let active = format!("{}/active", dir.trim_end_matches('/'));
    let names = disk.list(&active).map_err(StoreErr::Disk)?;
    if names
        .iter()
        .any(|n| n.ends_with(".bin") && Some(n) != allowed.as_ref())
    {
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
