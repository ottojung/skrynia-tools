use super::{abi, host::Host, path};

pub struct Abi;

impl Host for Abi {
    fn random_u32(&mut self) -> Result<u32, i32> {
        let mut bytes = [0u8; 4];
        let rc = unsafe { abi::secure_random_bits(bytes.as_mut_ptr() as u32, 32) };
        if rc < 0 {
            Err(rc)
        } else {
            Ok(u32::from_le_bytes(bytes))
        }
    }
    fn create_request(&mut self, ticket: u32) -> Result<i32, i32> {
        let p = path::ticket(ticket);
        let fd = unsafe { abi::open_disk(p.as_ptr() as u32, p.len() as u32, 15) };
        if fd < 0 { Err(fd) } else { Ok(fd) }
    }
    fn write_all(&mut self, fd: i32, bytes: &[u8]) -> Result<(), i32> {
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
    fn finish(&mut self, fd: i32) -> Result<(), i32> {
        let sync = unsafe { abi::sync(fd) };
        let close = unsafe { abi::close(fd) };
        if sync < 0 {
            Err(sync)
        } else if close < 0 {
            Err(close)
        } else {
            Ok(())
        }
    }
    fn remove_request(&mut self, ticket: u32) -> Result<(), i32> {
        super::sys::remove(ticket)
    }
    fn invoke(&mut self, publish: bool, ticket: u32) -> i32 {
        super::sys::invoke(publish, ticket)
    }
}
