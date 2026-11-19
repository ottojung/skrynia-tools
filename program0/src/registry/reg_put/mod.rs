mod lock;
mod mutate;
use super::cap::Disk;
use crate::MutLock;
use crate::reg::{RegErr, Registry};
use crate::store;
use crate::store_io;

impl Registry {
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
        let iid = store::invoke_id(admin_cap).to_vec();
        let code_hash = store::code_hash(&code);
        lock::with_prog_lock(lock, || {
            let cur = store_io::read_active(disk, &self.dir, &iid).map_err(RegErr::Store)?;
            if cur.is_some() {
                return Err(RegErr::Conflict);
            }
            store_io::ensure_blob(disk, &self.dir, &code).map_err(RegErr::Store)?;
            store_io::write_active(disk, &self.dir, &iid, &code_hash).map_err(RegErr::Store)?;
            Ok(())
        })?;
        let _ = code;
        Ok(iid)
    }
}
