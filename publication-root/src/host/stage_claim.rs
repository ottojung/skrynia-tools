//! Shared-RAM slot claim and release with `cas1` (wasm32 only).
use super::{abi, ram, stage::Staging};

impl Staging {
    /// Reserve occupancy bytes with `cas1` (0 -> 1); never a global mutex.
    pub(super) fn ready(&mut self) -> Result<i32, i32> {
        if let Some(fd) = self.fd {
            return Ok(fd);
        }
        let fd = unsafe { abi::open_ram() };
        if fd < 0 {
            return Err(fd);
        }
        let probe = unsafe { abi::cas1(fd, 0, 0, 0) };
        if probe == ram::ERR_BOUNDS {
            let rc = unsafe { abi::truncate(fd, ram::LIMIT) };
            if rc != 0 {
                return Err(rc);
            }
        } else if probe < 0 {
            return Err(probe);
        }
        let mut claimed = None;
        for byte in 0..ram::SLOTS {
            if unsafe { abi::cas1(fd, byte, 0, 1) } == 0 {
                claimed = Some(byte);
                break;
            }
        }
        let slot = claimed.ok_or(ram::ERR_IO)?;
        self.fd = Some(fd);
        self.slot = Some(slot);
        self.cursor = ram::Cursor::for_slot(slot);
        self.pos = 0;
        Ok(fd)
    }

    pub(super) fn release(&mut self) {
        if let (Some(fd), Some(slot)) = (self.fd, self.slot) {
            let _ = unsafe { abi::cas1(fd, slot, 1, 0) };
        }
        self.slot = None;
    }
}
