mod lock;
mod mutate;
mod releases;
use super::cap::Disk;
use crate::MutLock;
use crate::index;
use crate::reg::{RegErr, Registry};
use crate::store;
use crate::store_io;

impl Registry {
    #[allow(clippy::type_complexity)]
    pub fn register<D: Disk, L: MutLock>(
        &self,
        disk: &D,
        lock: &L,
        admin_cap: &[u8],
        code: Vec<u8>,
    ) -> Result<Vec<u8>, RegErr<D::Err, (), (), L::Err>> {
        if admin_cap.len() != 32 {
            return Err(RegErr::BadAuth);
        }
        if code.is_empty() || code.len() > 8 * 1024 * 1024 {
            return Err(RegErr::BadId);
        }
        let iid = store::invoke_id(admin_cap);
        let code_hash = store::code_hash(&code);
        lock::with_prog_lock(lock, || {
            index::recover(disk, &self.dir).map_err(RegErr::Store)?;
            index::ensure_program(disk, &self.dir).map_err(RegErr::Store)?;
            index::preflight(disk, index::scope(&self.dir)).map_err(RegErr::Store)?;
            index::dry_run(disk, index::scope(&self.dir), iid, true).map_err(RegErr::Store)?;
            let cur = store_io::read_active(disk, &self.dir, &iid).map_err(RegErr::Store)?;
            if cur.is_some() {
                return Err(RegErr::Conflict);
            }
            store_io::ensure_blob(disk, &self.dir, &code).map_err(RegErr::Store)?;
            store_io::seal_ready(disk, &self.dir, &iid, &code_hash).map_err(RegErr::Store)?;
            index::write_pending(disk, index::scope(&self.dir), iid).map_err(RegErr::Store)?;
            store_io::write_active(disk, &self.dir, &iid, &code_hash).map_err(RegErr::Store)?;
            index::insert(disk, index::scope(&self.dir), iid).map_err(RegErr::Store)?;
            index::clear_pending(disk, index::scope(&self.dir)).map_err(RegErr::Store)?;
            Ok(())
        })?;
        Ok(iid.to_vec())
    }
}
