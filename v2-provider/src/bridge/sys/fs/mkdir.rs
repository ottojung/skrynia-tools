use super::{Callback, InvokeError, path};
pub fn mkdir_p<I: Callback>(i: &I, name: &str) -> Result<(), InvokeError> {
    if name.is_empty() {
        return Ok(());
    }
    let mut cur = String::new();
    for part in name.split('/').filter(|p| !p.is_empty()) {
        if !cur.is_empty() {
            cur.push('/');
        }
        cur.push_str(part);
        let rc = path::mkdir(i, &cur, 4)?;
        if rc < 0 && path::is_dir(i, &cur)? != 1 {
            return Err(InvokeError::Unavailable);
        }
        if rc == 0
            && let Some((parent, _)) = cur.rsplit_once('/')
        {
            super::sync_dir(i, parent)?;
        }
    }
    Ok(())
}
