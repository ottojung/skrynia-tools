//! Mutation-lock guard for query-time index recovery and reads.
use super::res;
use crate::MutLock;

pub struct Guard<'a, L: MutLock>(&'a L);
impl<L: MutLock> Drop for Guard<'_, L> {
    fn drop(&mut self) {
        let _ = self.0.release();
    }
}

pub fn locked<L: MutLock>(lock: &L) -> Result<Guard<'_, L>, Vec<u8>> {
    match lock.try_acquire() {
        Ok(true) => Ok(Guard(lock)),
        Ok(false) => Err(res::mk_err(11)),
        Err(_) => Err(res::mk_err(7)),
    }
}
