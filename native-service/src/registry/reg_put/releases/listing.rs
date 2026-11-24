use super::*;
use crate::index;

impl Registry {
    pub fn list_ready<D: Disk>(
        &self,
        disk: &D,
        iid: &[u8],
        after: Option<[u8; 32]>,
        limit: u16,
    ) -> Result<Vec<[u8; 32]>, store_io::StoreErr<D::Err>> {
        if store_io::read_record(disk, &self.dir, iid)?.is_none() {
            return Err(store_io::StoreErr::Missing);
        }
        store_io::list_ready(disk, &self.dir, iid, after, limit)
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
        after: Option<[u8; 32]>,
        limit: u16,
    ) -> Result<Vec<[u8; 32]>, store_io::StoreErr<D::Err>> {
        index::recover(disk, &self.dir)?;
        index::page(disk, index::scope(&self.dir), &after, limit)
    }
}
