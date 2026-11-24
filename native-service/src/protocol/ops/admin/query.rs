use super::*;
use crate::{store, store_io::StoreErr, wire::AdminRequest};

pub fn handle<D: Disk>(reg: &Registry, disk: &D, req: &AdminRequest) -> Option<Vec<u8>> {
    Some(match req {
        AdminRequest::GetActive { invoke_id } => match reg.active_info(disk, invoke_id) {
            Ok(state) => res::ok(store::encode_active(&state)),
            Err(StoreErr::Missing) => res::mk_err(8),
            Err(StoreErr::BadId) => res::mk_err(9),
            Err(_) => res::mk_err(7),
        },
        AdminRequest::ListVersions {
            invoke_id,
            after,
            limit,
        } => {
            let Some(cursor) = super::page::after(after, *limit) else {
                return Some(res::mk_err(1));
            };
            match reg.list_ready(disk, invoke_id, Some(cursor), *limit) {
                Ok(hashes) => super::page::encode(&hashes),
                Err(StoreErr::Missing) => res::mk_err(8),
                Err(StoreErr::BadId) => res::mk_err(9),
                Err(_) => res::mk_err(7),
            }
        }
        AdminRequest::ListPrograms { after, limit } => {
            let Some(cursor) = super::page::after(after, *limit) else {
                return Some(res::mk_err(1));
            };
            match reg.list_programs(disk, Some(cursor), *limit) {
                Ok(ids) => super::page::encode(&ids),
                Err(_) => res::mk_err(7),
            }
        }
        _ => return None,
    })
}
