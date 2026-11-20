use super::{Callback, InvokeError, dir, file, path};
pub fn sync_dir<I: Callback>(i: &I, name: &str) -> Result<(), InvokeError> {
    let fd = dir::open(i, name)?;
    let rc = path::sync(i, fd)?;
    let _ = file::close(i, fd);
    if rc < 0 {
        Err(InvokeError::Unavailable)
    } else {
        Ok(())
    }
}
