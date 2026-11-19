mod app;
mod res;
use super::cap::{Disk, Eval, Random};
use crate::MutLock;
use crate::reg::Registry;
use crate::wire::Request;
use res::{map_res, ok as resp_ok};

pub fn dispatch<D: Disk, L: MutLock, R: Random, E: Eval>(
    reg: &Registry,
    disk: &D,
    lock: &L,
    _rng: &R,
    eval: &E,
    req: Request,
) -> Vec<u8> {
    if let Some(out) = app::handle(reg, disk, lock, eval, req.clone()) {
        return out;
    }
    match req {
        Request::Register { admin_cap, code } => {
            map_res(reg.register(disk, lock, &admin_cap, code).map(resp_ok), 2)
        }
        Request::Set {
            admin_cap,
            expect,
            code,
        } => map_res(
            reg.set(disk, lock, &admin_cap, &expect, code).map(resp_ok),
            3,
        ),
        Request::Revoke { admin_cap, expect } => map_res(
            reg.revoke(disk, lock, &admin_cap, &expect)
                .map(|_| resp_ok(Vec::new())),
            6,
        ),
        _ => unreachable!(),
    }
}
