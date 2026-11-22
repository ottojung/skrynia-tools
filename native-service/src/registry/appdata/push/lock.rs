use super::err::PushErr;
use crate::MutLock;

pub fn with_lock<D, L: MutLock, R, F>(lock: &L, f: F) -> Result<R, PushErr<D, L::Err>>
where
    F: FnOnce() -> Result<R, PushErr<D, L::Err>>,
{
    match lock.try_acquire().map_err(PushErr::Lock)? {
        true => {}
        false => return Err(PushErr::Busy),
    }
    struct Guard<'a, L: MutLock>(&'a L, bool);
    impl<L: MutLock> Drop for Guard<'_, L> {
        fn drop(&mut self) {
            if self.1 {
                let _ = self.0.release();
            }
        }
    }
    let mut guard = Guard(lock, true);
    let res = f();
    guard.1 = false;
    let rel = lock.release().map_err(PushErr::Lock);
    match (res, rel) {
        (Ok(v), Ok(())) => Ok(v),
        (Err(e), Ok(())) => Err(e),
        (Ok(_), Err(e)) => Err(e),
        (Err(e), Err(_)) => Err(e),
    }
}
