use super::{Disk, MutLock, RegErr, Registry, appdata};
use crate::wire::AppPrecond;
impl Registry {
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn app_put_wake<D: Disk, L: MutLock>(
        &self,
        disk: &D,
        lock: &L,
        write_cap: &[u8],
        key: &[u8],
        expect: &AppPrecond,
        value: &[u8],
        wake_grant: &[u8],
        wake_key: &[u8],
    ) -> Result<[u8; 32], RegErr<D::Err, (), (), L::Err>> {
        let auth = appdata::ops::WriteAuth::External(write_cap);
        appdata::ops::put_wake(
            disk, lock, &self.dir, auth, key, expect, value, wake_grant, wake_key,
        )
        .map_err(RegErr::from_app)
    }
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn app_delete_wake<D: Disk, L: MutLock>(
        &self,
        disk: &D,
        lock: &L,
        write_cap: &[u8],
        key: &[u8],
        expect: &[u8],
        wake_grant: &[u8],
        wake_key: &[u8],
    ) -> Result<(), RegErr<D::Err, (), (), L::Err>> {
        let auth = appdata::ops::WriteAuth::External(write_cap);
        appdata::ops::delete_wake(
            disk, lock, &self.dir, auth, key, expect, wake_grant, wake_key,
        )
        .map_err(RegErr::from_app)
    }
}
