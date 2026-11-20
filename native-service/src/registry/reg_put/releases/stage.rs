use super::*;

impl Registry {
    #[allow(clippy::type_complexity)]
    pub fn stage_release<D: Disk, L: MutLock>(
        &self,
        disk: &D,
        lock: &L,
        admin: &[u8],
        code: &[u8],
    ) -> Result<[u8; 32], RegErr<D::Err, (), (), L::Err>> {
        if code.is_empty() || code.len() > 8 * 1024 * 1024 {
            return Err(RegErr::BadId);
        }
        if admin.len() != 32 {
            return Err(RegErr::BadAuth);
        }
        let iid = store::invoke_id(admin).to_vec();
        with_prog_lock(lock, || {
            if store_io::read_active(disk, &self.dir, &iid)
                .map_err(RegErr::Store)?
                .is_none()
            {
                return Err(RegErr::Missing);
            }
            store_io::ensure_blob(disk, &self.dir, code).map_err(RegErr::Store)
        })
    }
}
