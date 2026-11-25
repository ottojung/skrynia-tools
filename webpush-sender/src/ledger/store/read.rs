use super::{Error, Result, MAX_LEDGER_BYTES};
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub(super) fn load(path: &Path) -> Result<Option<Vec<u8>>> {
    let file = match open(path)? {
        Some(file) => file,
        None => return Ok(None),
    };
    let meta = file.metadata().map_err(|_| Error::Transport("ledger-read"))?;
    if !meta.is_file() || meta.len() > MAX_LEDGER_BYTES {
        return Err(Error::Wire("ledger-shape"));
    }
    let mut bytes = Vec::with_capacity(meta.len() as usize);
    file.take(MAX_LEDGER_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| Error::Transport("ledger-read"))?;
    if bytes.len() as u64 > MAX_LEDGER_BYTES {
        return Err(Error::Wire("ledger-shape"));
    }
    Ok(Some(bytes))
}

fn open(path: &Path) -> Result<Option<File>> {
    let mut opts = std::fs::OpenOptions::new();
    opts.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.custom_flags(libc::O_CLOEXEC | libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    match opts.open(path) {
        Ok(file) => Ok(Some(file)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        #[cfg(unix)]
        Err(e) if e.raw_os_error() == Some(libc::ELOOP) => Err(Error::Wire("ledger-shape")),
        Err(_) => Err(Error::Transport("ledger-read")),
    }
}
