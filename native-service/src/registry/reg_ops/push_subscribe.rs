use super::Disk;
use super::appdata;
use super::map_push_err;
use crate::MutLock;
use crate::reg::{RegErr, Registry};

impl Registry {
    #[allow(clippy::type_complexity)]
    pub fn push_subscribe<D: Disk, L: MutLock>(
        &self,
        disk: &D,
        lock: &L,
        grant_cap: &[u8],
        key: &[u8],
        subscriber: &[u8],
        subscription: &[u8],
    ) -> Result<(), RegErr<D::Err, (), (), L::Err>> {
        appdata::push::subscribe(
            disk,
            lock,
            &self.dir,
            grant_cap,
            key,
            subscriber,
            subscription,
        )
        .map(|_| ())
        .map_err(map_push_err)
    }
}

impl Registry {
    pub fn push_sub_status<D: Disk, L: MutLock>(
        &self,
        disk: &D,
        lock: &L,
        grant_cap: &[u8],
        key: &[u8],
        subscriber: &[u8],
        expected: &[u8],
    ) -> Result<appdata::push::Status, crate::PushErr<D::Err, L::Err>> {
        appdata::push::sub_status(disk, lock, &self.dir, grant_cap, key, subscriber, expected)
    }
}
