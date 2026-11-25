use super::check::{MAX_CONFIG_BYTES, check};
use super::{Error, Result};
use std::io::Read;
use std::path::Path;

pub(super) fn read_file(path: &Path) -> Result<Vec<u8>> {
    let file = open(path)?;
    let meta = file.metadata().map_err(|_| Error::Config("config-read"))?;
    check(&meta)?;
    let mut bytes = Vec::with_capacity(meta.len() as usize);
    (&file)
        .take(MAX_CONFIG_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| Error::Config("config-read"))?;
    if bytes.len() as u64 > MAX_CONFIG_BYTES {
        return Err(Error::Config("config-size"));
    }
    Ok(bytes)
}

fn open(path: &Path) -> Result<std::fs::File> {
    let mut opts = std::fs::OpenOptions::new();
    opts.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.custom_flags(libc::O_CLOEXEC | libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    match opts.open(path) {
        Ok(f) => Ok(f),
        #[cfg(unix)]
        Err(e) if e.raw_os_error() == Some(libc::ELOOP) => Err(Error::Config("config-kind")),
        Err(_) => Err(Error::Config("config-read")),
    }
}
