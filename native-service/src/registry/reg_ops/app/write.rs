use super::{Disk, MutLock, RegErr, Registry, appdata};
use crate::wire::AppPrecond;

impl Registry {
    #[allow(clippy::type_complexity)]
    pub fn app_put<D: Disk, L: MutLock>(
        &self,
        disk: &D,
        lock: &L,
        write_cap: &[u8],
        key: &[u8],
        expect: &AppPrecond,
        value: &[u8],
    ) -> Result<[u8; 32], RegErr<D::Err, (), (), L::Err>> {
        let auth = appdata::ops::WriteAuth::External(write_cap);
        appdata::ops::put(disk, lock, &self.dir, auth, key, expect, value).map_err(RegErr::from_app)
    }

    #[allow(clippy::type_complexity)]
    pub fn app_delete<D: Disk, L: MutLock>(
        &self,
        disk: &D,
        lock: &L,
        write_cap: &[u8],
        key: &[u8],
        expect: &[u8],
    ) -> Result<(), RegErr<D::Err, (), (), L::Err>> {
        let auth = appdata::ops::WriteAuth::External(write_cap);
        appdata::ops::delete(disk, lock, &self.dir, auth, key, expect).map_err(RegErr::from_app)
    }
}
