use super::expect::match_expect;
use super::{Disk, MutLock, RegErr, Registry, with_prog_lock};
use crate::store;
use crate::store_io;
use crate::wire::AppPrecond;

impl Registry {
    pub fn revoke<D: Disk, L: MutLock>(
        &self,
        disk: &D,
        lock: &L,
        admin_cap: &[u8],
        expect: &AppPrecond,
    ) -> Result<(), RegErr<D::Err, (), (), L::Err>> {
        if admin_cap.len() != 32 {
            return Err(RegErr::BadAuth);
        }
        if let AppPrecond::Present(h) = expect {
            if h.len() != 32 {
                return Err(RegErr::BadAuth);
            }
        }
        let iid = store::invoke_id(admin_cap).to_vec();
        with_prog_lock(lock, || {
            let cur = store_io::read_active(disk, &self.dir, &iid).map_err(RegErr::Store)?;
            if !match_expect(cur, expect) {
                return Err(RegErr::Conflict);
            }
            store_io::remove_active(disk, &self.dir, &iid).map_err(RegErr::Store)?;
            Ok(())
        })?;
        self.code.write().unwrap().remove(&iid);
        Ok(())
    }
}
