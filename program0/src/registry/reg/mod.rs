mod err;
use super::cap::Disk;
use crate::store_io::StoreErr;

pub use err::RegErr;

pub struct Registry {
    pub(crate) dir: String,
}

impl Registry {
    pub fn open<D: Disk>(_disk: &D, dir: &str) -> Result<Self, StoreErr<D::Err>> {
        Ok(Self {
            dir: dir.to_string(),
        })
    }
}
