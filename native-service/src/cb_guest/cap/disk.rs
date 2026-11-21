use super::{disk_atom, disk_ls, disk_read, disk_rm};
use crate::cap::Disk;
pub struct CbDisk;
impl Disk for CbDisk {
    type Err = i32;
    fn read_all(&self, path: &str) -> Result<Option<Vec<u8>>, i32> {
        disk_read::read_all(path)
    }
    fn write_atomic(&self, path: &str, bytes: &[u8]) -> Result<(), i32> {
        disk_atom::write_atomic(path, bytes)
    }
    fn remove(&self, path: &str) -> Result<(), i32> {
        disk_rm::remove(path)
    }
    fn list(&self, dir: &str) -> Result<Vec<String>, i32> {
        disk_ls::list(dir)
    }
}
