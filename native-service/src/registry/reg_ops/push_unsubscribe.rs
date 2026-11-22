use super::Disk;
use super::appdata;
use super::map_push_err;
use crate::MutLock;
use crate::reg::{RegErr, Registry};

impl Registry {
    #[allow(clippy::type_complexity)]
    pub fn push_unsubscribe<D: Disk, L: MutLock>(
        &self,
        disk: &D,
        lock: &L,
        grant_cap: &[u8],
        key: &[u8],
        subscriber: &[u8],
    ) -> Result<(), RegErr<D::Err, (), (), L::Err>> {
        appdata::push::unsubscribe(disk, lock, &self.dir, grant_cap, key, subscriber)
            .map_err(map_push_err)
    }
}
