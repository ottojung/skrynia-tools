use super::*;

impl Registry {
    pub fn list_ready<D: Disk>(
        &self,
        disk: &D,
        iid: &[u8],
    ) -> Result<Vec<[u8; 32]>, store_io::StoreErr<D::Err>> {
        if store_io::read_record(disk, &self.dir, iid)?.is_none() {
            return Err(store_io::StoreErr::Missing);
        }
        store_io::list_ready(disk, &self.dir, iid)
    }
    pub fn active_info<D: Disk>(
        &self,
        disk: &D,
        iid: &[u8],
    ) -> Result<crate::store::ActiveState, store_io::StoreErr<D::Err>> {
        store_io::read_record(disk, &self.dir, iid)?.ok_or(store_io::StoreErr::Missing)
    }
    pub fn list_programs<D: Disk>(
        &self,
        disk: &D,
    ) -> Result<Vec<Vec<u8>>, store_io::StoreErr<D::Err>> {
        crate::store_ls::list_ids(disk, &self.dir)
    }
}
