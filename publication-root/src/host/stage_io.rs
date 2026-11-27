//! Byte staging and randomness inside a claimed shared-RAM slot (wasm32 only).
use super::{abi, stage::Staging};

impl Staging {
    /// Reserve a fresh region inside this slot, write it byte-by-byte.
    pub fn put(&mut self, bytes: &[u8]) -> Result<(u64, u64), i32> {
        self.ready()?;
        let at = self.cursor.alloc(bytes.len() as u64)?;
        let fd = self.seek_to(at)?;
        for &byte in bytes {
            let rc = unsafe { abi::write1(fd, byte as u32) };
            if rc != 0 {
                return Err(rc);
            }
        }
        self.pos = at + bytes.len() as u64;
        Ok((at, bytes.len() as u64))
    }

    /// Four random bytes through shared RAM, never through linear memory.
    pub fn random(&mut self, out: &mut [u8; 4]) -> Result<(), i32> {
        self.ready()?;
        let at = self.cursor.alloc(4)?;
        let rc = unsafe { abi::secure_random_bits(at, 32) };
        if rc != 0 {
            return Err(rc);
        }
        let fd = self.seek_to(at)?;
        for slot in out.iter_mut() {
            let byte = unsafe { abi::read1(fd) };
            if byte < 0 {
                return Err(byte);
            }
            *slot = byte as u8;
        }
        self.pos = at + 4;
        Ok(())
    }
}
