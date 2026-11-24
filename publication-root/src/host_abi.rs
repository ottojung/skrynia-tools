use super::abi;
use super::host::{Abi, Host};

impl Host for Abi {
    fn open_read(&mut self, path: &[u8]) -> Result<i32, i32> {
        let fd = unsafe { abi::open_disk(path.as_ptr() as u32, path.len() as u32, 1) };
        if fd < 0 { Err(fd) } else { Ok(fd) }
    }
    fn read(&mut self, fd: i32, dst: &mut [u8]) -> Result<usize, i32> {
        let n = unsafe { abi::read(fd, dst.as_mut_ptr() as u32, dst.len() as u32) };
        if n < 0 { Err(n) } else { Ok(n as usize) }
    }
    fn close(&mut self, fd: i32) -> Result<(), i32> {
        let rc = unsafe { abi::close(fd) };
        if rc < 0 { Err(rc) } else { Ok(()) }
    }
    fn mkdir(&mut self, path: &[u8]) -> Result<(), i32> {
        let rc = unsafe { abi::mkdir(path.as_ptr() as u32, path.len() as u32, 4) };
        if rc == 0 || rc == -3 { Ok(()) } else { Err(rc) }
    }
    fn random(&mut self) -> Result<u32, i32> {
        let mut b = [0u8; 4];
        let rc = unsafe { abi::secure_random_bits(b.as_mut_ptr() as u32, 32) };
        if rc < 0 {
            Err(rc)
        } else {
            Ok(u32::from_le_bytes(b))
        }
    }
    fn create(&mut self, path: &[u8]) -> Result<i32, i32> {
        let fd = unsafe { abi::open_disk(path.as_ptr() as u32, path.len() as u32, 15) };
        if fd < 0 { Err(fd) } else { Ok(fd) }
    }
    fn write(&mut self, fd: i32, bytes: &[u8]) -> Result<(), i32> {
        let mut at = 0;
        while at < bytes.len() {
            let n =
                unsafe { abi::write(fd, bytes[at..].as_ptr() as u32, (bytes.len() - at) as u32) };
            if n <= 0 {
                return Err(if n == 0 { -5 } else { n });
            }
            at += n as usize;
        }
        Ok(())
    }
    fn sync(&mut self, fd: i32) -> Result<(), i32> {
        let rc = unsafe { abi::sync(fd) };
        if rc < 0 { Err(rc) } else { Ok(()) }
    }
    fn delete(&mut self, path: &[u8]) -> Result<(), i32> {
        let rc = unsafe { abi::delete(path.as_ptr() as u32, path.len() as u32) };
        if rc == 0 || rc == -2 { Ok(()) } else { Err(rc) }
    }
    fn invoke(&mut self, publish: bool, ticket: u32) -> Option<i32> {
        // A trap here means the effect may have happened; the caller sees no
        // response and must treat the operation as UNKNOWN, never as success.
        Some(unsafe {
            if publish {
                abi::publish(ticket)
            } else {
                abi::unpublish(ticket)
            }
        })
    }
}
