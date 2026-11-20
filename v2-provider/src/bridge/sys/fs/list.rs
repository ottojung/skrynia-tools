use super::{Callback, InvokeError, dir, file};
pub fn list_dir<I: Callback>(i: &I, name: &str) -> Result<Vec<String>, InvokeError> {
    let fd = dir::open(i, name)?;
    let mut out = Vec::new();
    loop {
        let (st, data) = dir::next(i, fd)?;
        if st == 0 {
            break;
        }
        if st < 0 {
            let _ = file::close(i, fd);
            return Err(InvokeError::Unavailable);
        }
        out.push(String::from_utf8(data).map_err(|_| InvokeError::Unavailable)?);
    }
    let _ = file::close(i, fd);
    Ok(out)
}
