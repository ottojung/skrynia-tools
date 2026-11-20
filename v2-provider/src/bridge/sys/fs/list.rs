use super::{Callback, InvokeError, dir, file};
pub fn list_dir<I: Callback>(i: &I, name: &str) -> Result<Vec<String>, InvokeError> {
    let fd = dir::open(i, name)?;
    let result = (|| {
        let mut out = Vec::new();
        loop {
            let (st, data) = dir::next(i, fd)?;
            if st < 0 {
                return Err(InvokeError::Unavailable);
            }
            if st == 0 {
                return Ok(out);
            }
            if st as usize != data.len() {
                return Err(InvokeError::Unavailable);
            }
            out.push(String::from_utf8(data).map_err(|_| InvokeError::Unavailable)?);
        }
    })();
    let closed = file::close(i, fd);
    if closed != Ok(0) {
        return Err(InvokeError::Unavailable);
    }
    result
}
