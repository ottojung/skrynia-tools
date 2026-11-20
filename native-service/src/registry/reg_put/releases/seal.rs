use super::*;

impl Registry {
    #[allow(clippy::type_complexity)]
    pub fn seal_release<D: Disk, L: MutLock>(
        &self,
        disk: &D,
        lock: &L,
        admin: &[u8],
        hash: &[u8; 32],
    ) -> Result<(), RegErr<D::Err, (), (), L::Err>> {
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
            store_io::seal_ready(disk, &self.dir, &iid, hash).map_err(RegErr::Store)
        })
    }
}
