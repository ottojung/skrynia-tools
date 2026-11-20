use skrynia_native_service::cap::{Disk, Eval, Random};
use skrynia_native_service::reg::Registry;
use skrynia_native_service::store_io::StoreErr;
use skrynia_native_service::{MutLock, handle};

pub fn open<D: Disk>(disk: &D, dir: &str) -> Result<Registry, StoreErr<D::Err>> {
    Registry::open(disk, dir)
}

pub fn dispatch<D, L, R, E>(
    reg: &Registry,
    disk: &D,
    lock: &L,
    rng: &R,
    eval: &E,
    req: &[u8],
) -> Vec<u8>
where
    D: Disk,
    L: MutLock,
    R: Random,
    E: Eval,
{
    handle::handle(reg, disk, lock, rng, eval, req)
}
