use super::push::PushErr;
use super::{AppErr, MutLock, State};
use crate::hash_bytes;
use crate::wire::AppPrecond;
pub fn hint_err<D, L>(e: PushErr<D, ()>) -> AppErr<D, L> {
    match e {
        PushErr::Disk(d) => AppErr::Disk(d),
        PushErr::Busy | PushErr::Quota => AppErr::Busy,
        _ => AppErr::BadAuth,
    }
}

pub fn matches_precond(state: &State, expect: &AppPrecond) -> bool {
    match (state, expect) {
        (State::Absent, AppPrecond::Absent) => true,
        (State::Present(v), AppPrecond::Present(h)) => hash_bytes(v).as_slice() == h.as_slice(),
        _ => false,
    }
}

struct Guard<'a, L: MutLock>(&'a L, bool);
impl<L: MutLock> Drop for Guard<'_, L> {
    fn drop(&mut self) {
        if self.1 {
            let _ = self.0.release();
        }
    }
}

pub fn with_lock<D, L: MutLock, R, F>(lock: &L, f: F) -> Result<R, AppErr<D, L::Err>>
where
    F: FnOnce() -> Result<R, AppErr<D, L::Err>>,
{
    match lock.try_acquire().map_err(AppErr::Lock)? {
        true => {}
        false => return Err(AppErr::Busy),
    }
    let mut guard = Guard(lock, true);
    let res = f();
    guard.1 = false;
    let rel = lock.release().map_err(AppErr::Lock);
    match (res, rel) {
        (Ok(v), Ok(())) => Ok(v),
        (Err(e), Ok(())) => Err(e),
        (Ok(_), Err(e)) => Err(e),
        (Err(e), Err(_)) => Err(e),
    }
}
