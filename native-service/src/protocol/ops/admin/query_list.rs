//! Locked LIST_VERSIONS / LIST_PROGRAMS request handling.
use super::lock::locked;
use super::{Disk, MutLock, Registry, page, res};
use crate::store_io::StoreErr;
use crate::wire::AdminRequest;

fn cursor(req: &AdminRequest) -> Result<super::page::Cursor, Vec<u8>> {
    match req {
        AdminRequest::ListVersions { after, limit, .. }
        | AdminRequest::ListPrograms { after, limit } => {
            page::after(after, *limit).ok_or_else(|| res::mk_err(1))
        }
        _ => Err(res::mk_err(1)),
    }
}

pub fn handle<D: Disk, L: MutLock>(
    reg: &Registry,
    disk: &D,
    lock: &L,
    req: &AdminRequest,
) -> Option<Vec<u8>> {
    let c = cursor(req).ok()?;
    let guard = locked(lock).ok()?;
    let out = match req {
        AdminRequest::ListVersions {
            invoke_id, limit, ..
        } => match reg.list_ready(disk, invoke_id, c, *limit) {
            Ok(v) => page::encode(&v),
            Err(StoreErr::Missing) => res::mk_err(8),
            Err(StoreErr::BadId) => res::mk_err(9),
            Err(_) => res::mk_err(7),
        },
        AdminRequest::ListPrograms { limit, .. } => match reg.list_programs(disk, c, *limit) {
            Ok(v) => page::encode(&v),
            Err(_) => res::mk_err(7),
        },
        _ => return None,
    };
    drop(guard);
    Some(out)
}
