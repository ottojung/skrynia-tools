use super::expect::match_expect;
use super::{Disk, MutLock, RegErr, Registry, with_prog_lock};
use crate::index;
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
        if let AppPrecond::Present(h) = expect
            && h.len() != 32
        {
            return Err(RegErr::BadAuth);
        }
        let iid = store::invoke_id(admin_cap);
        with_prog_lock(lock, || {
            index::recover(disk, &self.dir).map_err(RegErr::Store)?;
            let cur = store_io::read_active(disk, &self.dir, &iid).map_err(RegErr::Store)?;
            if !match_expect(cur, expect) {
                return Err(RegErr::Conflict);
            }
            index::preflight(disk, index::scope(&self.dir)).map_err(RegErr::Store)?;
            index::dry_run(disk, index::scope(&self.dir), iid, false).map_err(RegErr::Store)?;
            index::write_pending(disk, index::scope(&self.dir), iid).map_err(RegErr::Store)?;
            store_io::remove_active(disk, &self.dir, &iid).map_err(RegErr::Store)?;
            index::remove(disk, index::scope(&self.dir), iid).map_err(RegErr::Store)?;
            index::clear_pending(disk, index::scope(&self.dir)).map_err(RegErr::Store)?;
            Ok(())
        })?;
        Ok(())
    }
}
