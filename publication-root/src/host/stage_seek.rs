//! Signed relative seek inside a claimed shared-RAM slot (wasm32 only).
use super::{abi, ram, stage::Staging};

impl Staging {
    /// `seek` is a signed displacement from the current RAM fd cursor.
    pub(super) fn seek_to(&mut self, target: u64) -> Result<i32, i32> {
        let fd = self.ready()?;
        let delta = target as i64 - self.pos as i64;
        let got = unsafe { abi::seek(fd, delta) };
        if got < 0 {
            return Err(got as i32);
        }
        if got as u64 != target {
            return Err(ram::ERR_IO);
        }
        self.pos = got as u64;
        Ok(fd)
    }
}
