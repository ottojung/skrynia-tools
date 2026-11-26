//! Byte-only staging into a CAS-claimed shared-RAM slot (wasm32 only).
use super::ram::Cursor;

pub struct Staging {
    pub(super) fd: Option<i32>,
    pub(super) pos: u64,
    pub(super) slot: Option<u64>,
    pub(super) cursor: Cursor,
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
            if unsafe { super::abi::seek(fd, -(self.pos as i64)) } >= 0 {
                self.pos = 0;
            }
        } else {
            self.pos = 0;
        }
    }
}

impl Drop for Staging {
    fn drop(&mut self) {
        self.release();
    }
}
