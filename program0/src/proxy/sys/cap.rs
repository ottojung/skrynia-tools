use skrynia_native_service::{Disk, MutLock};
pub struct HostDisk;
impl Disk for HostDisk {
    type Err = i32;
    fn read_all(&self, path: &str) -> Result<Option<Vec<u8>>, i32> {
        super::disk::raw_read(path)
    }
    fn write_atomic(&self, path: &str, bytes: &[u8]) -> Result<(), i32> {
        super::disk_io::raw_write(path, bytes)
    }
    fn remove(&self, path: &str) -> Result<(), i32> {
        super::disk_io::raw_remove(path)
    }
    fn list(&self, dir: &str) -> Result<Vec<String>, i32> {
        super::disk_io::raw_list(dir)
    }
}
pub struct HostLock;
impl MutLock for HostLock {
    type Err = i32;
    fn try_acquire(&self) -> Result<bool, i32> {
        let n = unsafe { super::super::abi::try_mutation_lock() };
        if n == 0 {
            Ok(true)
        } else if n == -3 {
            Ok(false)
        } else {
            Err(n)
        }
    }
    fn release(&self) -> Result<(), i32> {
        let n = unsafe { super::super::abi::release_mutation_lock() };
        if n < 0 { Err(n) } else { Ok(()) }
    }
}
