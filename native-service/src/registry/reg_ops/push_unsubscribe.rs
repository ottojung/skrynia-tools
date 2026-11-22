use super::Disk;
use super::appdata;
use crate::MutLock;
use crate::PushErr;
use crate::reg::{RegErr, Registry};

impl Registry {
    #[allow(clippy::type_complexity)]
    pub fn push_unsubscribe<D: Disk, L: MutLock>(
        &self,
        disk: &D,
        lock: &L,
        write_cap: &[u8],
        key: &[u8],
        subscriber: &[u8],
    ) -> Result<(), RegErr<D::Err, (), (), L::Err>> {
        appdata::push::unsubscribe(disk, lock, &self.dir, write_cap, key, subscriber)
            .map_err(map_push_err)
    }
}

fn map_push_err<D, L>(e: PushErr<D, L>) -> RegErr<D, (), (), L> {
    match e {
        PushErr::Disk(d) => RegErr::Store(crate::store_io::StoreErr::Disk(d)),
        PushErr::Lock(l) => RegErr::Lock(l),
        PushErr::BadAuth | PushErr::BadKey => RegErr::BadAuth,
        PushErr::Missing => RegErr::Missing,
        PushErr::Busy => RegErr::Busy,
        PushErr::Quota => RegErr::Conflict,
    }
}
