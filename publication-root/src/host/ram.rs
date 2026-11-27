//! Shared-RAM staging slots: bounded, disjoint, lock-free (no global mutex).
//!
//! Bytes 0..SLOTS are one CAS occupancy byte each (0 free, 1 claimed). A
//! request claims a distinct byte, then stages inside its own 8 KiB slot, so
//! concurrent userland invocations never overwrite each other's staged bytes.

#[cfg(target_arch = "wasm32")]
pub const SLOTS: u64 = 256;
pub const BASE: u64 = 256;
pub const SLOT: u64 = 8192;
#[cfg(target_arch = "wasm32")]
pub const LIMIT: u64 = BASE + SLOT * SLOTS;
pub const ERR_RANGE: i32 = -6;
pub const ERR_BOUNDS: i32 = -7;
#[cfg(target_arch = "wasm32")]
pub const ERR_IO: i32 = -5;

#[derive(Clone, Copy)]
pub struct Cursor {
    base: u64,
    next: u64,
    end: u64,
}

impl Cursor {
    pub const fn new(base: u64, len: u64) -> Self {
        Self {
            base,
            next: base,
            end: base + len,
        }
    }

    pub const fn for_slot(slot: u64) -> Self {
        Self::new(BASE + slot * SLOT, SLOT)
    }

    pub fn alloc(&mut self, len: u64) -> Result<u64, i32> {
        let end = self.next.checked_add(len).ok_or(ERR_RANGE)?;
        if end > self.end {
            return Err(ERR_BOUNDS);
        }
        let at = self.next;
        self.next = end;
        Ok(at)
    }

    pub fn reset(&mut self) {
        self.next = self.base;
    }

    #[cfg(test)]
    pub fn next(&self) -> u64 {
        self.next
    }

    #[cfg(test)]
    pub fn base(&self) -> u64 {
        self.base
    }
}
