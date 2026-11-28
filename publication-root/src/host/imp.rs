use super::{Abi, Host, abi, fs, ops};

impl Host for Abi {
    fn open(&mut self, path: &[u8], mode: u32) -> Result<i32, i32> {
        ops::open(&mut self.staging, path, mode)
    }
    fn exists(&mut self, path: &[u8]) -> bool {
        let Ok((ptr, len)) = self.staging.put(path) else {
            return false;
        };
        unsafe { abi::exists(ptr, len) == 1 }
    }
    fn read(&mut self, fd: i32, dst: &mut [u8]) -> Result<usize, i32> {
        ops::read(fd, dst)
    }
    fn close(&mut self, fd: i32) -> Result<(), i32> {
        ops::close(fd)
    }
    fn mkdir(&mut self, path: &[u8]) -> Result<(), i32> {
        ops::mkdir(&mut self.staging, path)
    }
    fn random(&mut self) -> Result<u32, i32> {
        ops::random(&mut self.staging)
    }
    fn write(&mut self, fd: i32, bytes: &[u8]) -> Result<(), i32> {
        fs::write(fd, bytes)
    }
    fn delete(&mut self, path: &[u8]) -> Result<(), i32> {
        fs::delete(&mut self.staging, path)
    }
    fn invoke(&mut self, publish: bool, ticket: u32) -> Option<i32> {
        fs::invoke(publish, ticket)
    }
}
