//! Admin query dispatch: active read is immutable; list paths are locked.
use super::*;
use crate::{store, store_io::StoreErr, wire::AdminRequest};

pub fn handle<D: Disk, L: MutLock>(
    reg: &Registry,
    disk: &D,
    lock: &L,
    req: &AdminRequest,
) -> Option<Vec<u8>> {
    match req {
        AdminRequest::GetActive { invoke_id } => Some(match reg.active_info(disk, invoke_id) {
            Ok(state) => res::ok(store::encode_active(&state)),
            Err(StoreErr::Missing) => res::mk_err(8),
            Err(StoreErr::BadId) => res::mk_err(9),
            Err(_) => res::mk_err(7),
        }),
        AdminRequest::ListVersions { .. } | AdminRequest::ListPrograms { .. } => {
            super::query_list::handle(reg, disk, lock, req)
        }
        _ => None,
    }
}
