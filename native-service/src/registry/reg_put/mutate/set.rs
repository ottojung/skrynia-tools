use super::expect::match_expect;
use super::{Disk, MutLock, RegErr, Registry, with_prog_lock};
use crate::store;
use crate::store_io;
use crate::wire::AppPrecond;

impl Registry {
    #[allow(clippy::type_complexity)]
    pub fn set<D: Disk, L: MutLock>(
        &self,
        disk: &D,
        lock: &L,
        admin_cap: &[u8],
        expect: &AppPrecond,
        code: Vec<u8>,
    ) -> Result<Vec<u8>, RegErr<D::Err, (), (), L::Err>> {
        if admin_cap.len() != 32 {
            return Err(RegErr::BadAuth);
        }
        if code.is_empty() || code.len() > 8 * 1024 * 1024 {
            return Err(RegErr::BadId);
        }
        let AppPrecond::Present(h) = expect else {
            return Err(RegErr::BadAuth);
        };
        if h.len() != 32 {
            return Err(RegErr::BadAuth);
        }
        let iid = store::invoke_id(admin_cap).to_vec();
        let new_hash = store::code_hash(&code);
        with_prog_lock(lock, || {
            let cur = store_io::read_active(disk, &self.dir, &iid).map_err(RegErr::Store)?;
            if !match_expect(cur, expect) {
                return Err(RegErr::Conflict);
            }
            store_io::ensure_blob(disk, &self.dir, &code).map_err(RegErr::Store)?;
            store_io::seal_ready(disk, &self.dir, &iid, &new_hash).map_err(RegErr::Store)?;
            store_io::write_active(disk, &self.dir, &iid, &new_hash).map_err(RegErr::Store)?;
            Ok(())
        })?;
        let _ = code;
        Ok(new_hash.to_vec())
    }
}
