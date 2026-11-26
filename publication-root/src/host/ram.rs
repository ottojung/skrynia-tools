//! Shared-RAM staging slots: bounded, disjoint, lock-free (no global mutex).
//!
//! Byte 0 is a sequentially-consistent slot counter claimed with `cas1`.
//! Every invocation takes a distinct 8 KiB slot, so concurrent userland
//! requests never overwrite each other's staged path or random bytes.

#[cfg(target_arch = "wasm32")]
pub const COUNTER: u64 = 0;
pub const BASE: u64 = 1;
pub const SLOT: u64 = 8192;
#[cfg(target_arch = "wasm32")]
pub const SLOTS: u64 = 256;
#[cfg(target_arch = "wasm32")]
pub const LIMIT: u64 = BASE + SLOT * SLOTS;
pub const ERR_RANGE: i32 = -6;
pub const ERR_BOUNDS: i32 = -7;

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

#[cfg(test)]
mod tests {
    use super::{BASE, Cursor, ERR_BOUNDS, ERR_RANGE, SLOT};

    #[test]
    fn distinct_slots_are_disjoint() {
        let mut a = Cursor::for_slot(0);
        let mut b = Cursor::for_slot(1);
        assert_eq!(a.base(), BASE);
        assert_eq!(b.base(), BASE + SLOT);
        assert_eq!(a.alloc(8).unwrap(), BASE);
        assert_eq!(b.alloc(8).unwrap(), BASE + SLOT);
        assert_eq!(a.alloc(u64::MAX), Err(ERR_RANGE));
        assert_eq!(a.alloc(SLOT), Err(ERR_BOUNDS));
    }

    #[test]
    fn reset_reclaims_only_its_own_slot() {
        let mut c = Cursor::for_slot(3);
        assert_eq!(c.alloc(4096).unwrap(), BASE + 3 * SLOT);
        c.reset();
        assert_eq!(c.alloc(4).unwrap(), BASE + 3 * SLOT);
        assert_eq!(c.next(), BASE + 3 * SLOT + 4);
    }
}
