use crate::MutLock;
use crate::reg::RegErr;

pub fn with_prog_lock<D, L: MutLock, R, F>(lock: &L, f: F) -> Result<R, RegErr<D, (), (), L::Err>>
where
    F: FnOnce() -> Result<R, RegErr<D, (), (), L::Err>>,
{
    match lock.try_acquire().map_err(RegErr::Lock)? {
        true => {}
        false => return Err(RegErr::Busy),
    }
    let res = f();
    let rel = lock.release().map_err(RegErr::Lock);
    match (res, rel) {
        (Ok(v), Ok(())) => Ok(v),
        (Err(e), Ok(())) => Err(e),
        (Ok(_), Err(e)) => Err(e),
        (Err(e), Err(_)) => Err(e),
    }
}
