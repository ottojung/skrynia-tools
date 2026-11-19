use super::*;
use crate::{store, wire::Request};

pub fn handle<D: Disk, L: MutLock>(
    reg: &Registry,
    disk: &D,
    lock: &L,
    req: &Request,
) -> Option<Vec<u8>> {
    Some(match req {
        Request::StageCode { admin_cap, code } => res::map_res(
            reg.stage_release(disk, lock, admin_cap, code)
                .map(|h| res::ok(h.to_vec())),
            7,
        ),
        Request::SealCode { admin_cap, hash } => {
            let Ok(hash): Result<[u8; 32], _> = hash.as_slice().try_into() else {
                return Some(res::mk_err(1));
            };
            res::map_res(
                reg.seal_release(disk, lock, admin_cap, &hash)
                    .map(|()| res::ok(Vec::new())),
                7,
            )
        }
        Request::ActivateCode {
            admin_cap,
            expected_revision,
            target,
        } => {
            let Ok(hash): Result<[u8; 32], _> = target.as_slice().try_into() else {
                return Some(res::mk_err(1));
            };
            res::map_res(
                reg.activate_release(disk, lock, admin_cap, *expected_revision, &hash)
                    .map(|s| res::ok(store::encode_active(&s))),
                7,
            )
        }
        _ => return None,
    })
}
