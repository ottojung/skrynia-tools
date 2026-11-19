use super::*;
use crate::{store, store_io::StoreErr, wire::Request};

pub fn handle<D: Disk>(reg: &Registry, disk: &D, req: &Request) -> Option<Vec<u8>> {
    Some(match req {
        Request::GetActive { invoke_id } => match reg.active_info(disk, invoke_id) {
            Ok(state) => res::ok(store::encode_active(&state)),
            Err(StoreErr::Missing) => res::mk_err(8),
            Err(StoreErr::BadId) => res::mk_err(9),
            Err(_) => res::mk_err(7),
        },
        Request::ListVersions {
            invoke_id,
            after,
            limit,
        } => match reg.list_ready(disk, invoke_id) {
            Ok(hashes) => super::page::page(hashes, after, *limit),
            Err(StoreErr::Missing) => res::mk_err(8),
            Err(StoreErr::BadId) => res::mk_err(9),
            Err(_) => res::mk_err(7),
        },
        Request::ListPrograms { after, limit } => match reg.list_programs(disk) {
            Ok(ids) => {
                let hashes = ids.into_iter().map(|id| id.try_into()).collect();
                match hashes {
                    Ok(v) => super::page::page(v, after, *limit),
                    Err(_) => res::mk_err(7),
                }
            }
            Err(_) => res::mk_err(7),
        },
        _ => return None,
    })
}
