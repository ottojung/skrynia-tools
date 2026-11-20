use super::cap::{Disk, Eval, Random};
use super::ops;
use super::res::err;
use crate::MutLock;
use crate::reg::Registry;
use crate::wdec::decode_req;

pub fn handle<D: Disk, L: MutLock, R: Random, E: Eval>(
    reg: &Registry,
    disk: &D,
    lock: &L,
    rng: &R,
    eval: &E,
    req: &[u8],
) -> Vec<u8> {
    let req = match decode_req(req) {
        Ok(r) => r,
        Err(_) => return err(1),
    };
    ops::dispatch(reg, disk, lock, rng, eval, req)
}
