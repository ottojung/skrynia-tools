use super::{Disk, RegErr, Registry, appdata};

impl Registry {
    pub fn app_get<D: Disk>(
        &self,
        disk: &D,
        read_cap: &[u8],
        key: &[u8],
    ) -> Result<Vec<u8>, RegErr<D::Err, (), (), ()>> {
        appdata::ops::get(disk, &self.dir, read_cap, key).map_err(RegErr::from_app)
    }
}
