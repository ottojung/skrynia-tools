use super::{Callback, InvokeError, file, io, mkdir_p, path, rand, sync_dir};
pub fn write_atomic<I: Callback>(i: &I, name: &str, bytes: &[u8]) -> Result<(), InvokeError> {
    let (parent, basename) = name.rsplit_once('/').ok_or(InvokeError::Unavailable)?;
    mkdir_p(i, parent)?;
    let rnd = rand::bytes(i, 8)?;
    let tmp = format!(
        "{parent}/.{basename}.tmp.{:016x}",
        u64::from_le_bytes(rnd.try_into().map_err(|_| InvokeError::Unavailable)?)
    );
    let fd = file::open_disk(i, &tmp, 2 + 4 + 8 + 16)?;
    let saved = io::write_all_fd(i, fd, bytes).and_then(|_| {
        if path::sync(i, fd)? == 0 {
            Ok(())
        } else {
            Err(InvokeError::Unavailable)
        }
    });
    let closed = file::close(i, fd);
    if saved.is_err() || closed != Ok(0) {
        let _ = path::delete(i, &tmp);
        let _ = sync_dir(i, parent);
        return Err(InvokeError::Unavailable);
    }
    if path::rename(i, &tmp, name)? != 0 {
        let _ = path::delete(i, &tmp);
        let _ = sync_dir(i, parent);
        return Err(InvokeError::Unavailable);
    }
    sync_dir(i, parent)
}
