//! Byte-only staging into a CAS-claimed shared-RAM slot (wasm32 only).
use super::{abi, ram::Cursor};

pub struct Staging {
    fd: Option<i32>,
    pos: u64,
    slot: Option<u64>,
    cursor: Cursor,
}

impl Staging {
    pub const fn new() -> Self {
        Self {
            fd: None,
            pos: 0,
            slot: None,
            cursor: Cursor::for_slot(0),
        }
    }

    pub fn reset(&mut self) {
        self.cursor.reset();
        if let Some(fd) = self.fd {
            if unsafe { abi::seek(fd, -(self.pos as i64)) } >= 0 {
                self.pos = 0;
            }
        } else {
            self.pos = 0;
        }
    }

    /// Reserve occupancy bytes with `cas1` (0 -> 1); never a global mutex.
    fn ready(&mut self) -> Result<i32, i32> {
        if let Some(fd) = self.fd {
            return Ok(fd);
        }
        let fd = unsafe { abi::open_ram() };
        if fd < 0 {
            return Err(fd);
        }
        let probe = unsafe { abi::cas1(fd, 0, 0, 0) };
        if probe == super::ram::ERR_BOUNDS {
            let rc = unsafe { abi::truncate(fd, super::ram::LIMIT) };
            if rc != 0 {
                return Err(rc);
            }
        } else if probe < 0 {
            return Err(probe);
        }
        let mut claimed = None;
        for byte in 0..super::ram::SLOTS {
            if unsafe { abi::cas1(fd, byte, 0, 1) } == 0 {
                claimed = Some(byte);
                break;
            }
        }
        let slot = claimed.ok_or(super::ram::ERR_IO)?;
        self.fd = Some(fd);
        self.slot = Some(slot);
        self.cursor = Cursor::for_slot(slot);
        self.pos = 0;
        Ok(fd)
    }

    pub fn release(&mut self) {
        if let (Some(fd), Some(slot)) = (self.fd, self.slot) {
            let _ = unsafe { abi::cas1(fd, slot, 1, 0) };
        }
        self.slot = None;
    }

    /// `seek` is a signed displacement from the current RAM fd cursor.
    fn seek_to(&mut self, target: u64) -> Result<i32, i32> {
        let fd = self.ready()?;
        let delta = target as i64 - self.pos as i64;
        let got = unsafe { abi::seek(fd, delta) };
        if got < 0 {
            return Err(got as i32);
        }
        if got as u64 != target {
            return Err(super::ram::ERR_IO);
        }
        self.pos = got as u64;
        Ok(fd)
    }

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

impl Drop for Staging {
    fn drop(&mut self) {
        self.release();
    }
}
