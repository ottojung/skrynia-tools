mod mutate;
mod page;
mod query;
use super::res;
use super::{Disk, MutLock, Registry};
use crate::wire::Request;

pub fn handle<D: Disk, L: MutLock>(
    reg: &Registry,
    disk: &D,
    lock: &L,
    req: Request,
) -> Option<Vec<u8>> {
    if let Some(v) = mutate::handle(reg, disk, lock, &req) {
        return Some(v);
    }
    query::handle(reg, disk, &req)
}
