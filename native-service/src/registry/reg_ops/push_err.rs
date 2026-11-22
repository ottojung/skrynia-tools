//! Single mapping from P1 push failures to registry errors.
use crate::PushErr;
use crate::reg::RegErr;

pub(crate) fn map_push_err<D, L>(e: PushErr<D, L>) -> RegErr<D, (), (), L> {
    match e {
        PushErr::Disk(d) => RegErr::Store(crate::store_io::StoreErr::Disk(d)),
        PushErr::Lock(l) => RegErr::Lock(l),
        PushErr::BadAuth | PushErr::BadKey => RegErr::BadAuth,
        PushErr::Missing => RegErr::Missing,
        PushErr::Busy => RegErr::Busy,
        PushErr::Quota => RegErr::Conflict,
    }
}
