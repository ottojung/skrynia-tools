use super::{Callback, InvokeError, dir, file, path};
pub fn sync_dir<I: Callback>(i: &I, name: &str) -> Result<(), InvokeError> {
    let fd = dir::open(i, name)?;
    let sync_result = path::sync(i, fd);
    let closed = file::close(i, fd);
    if sync_result != Ok(0) || closed != Ok(0) {
        return Err(InvokeError::Unavailable);
    }
    Ok(())
}
