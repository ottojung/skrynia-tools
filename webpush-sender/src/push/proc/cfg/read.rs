use super::{Error, Result};
use std::io::Read;
use std::path::Path;
const MAX_CONFIG_BYTES: u64 = 64 * 1024;

pub(super) fn read_file(path: &Path) -> Result<Vec<u8>> {
    let mut opts = std::fs::OpenOptions::new();
    opts.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.custom_flags(libc::O_CLOEXEC | libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    let file = opts.open(path).map_err(|_| Error::Config("config-read"))?;
    let meta = file.metadata().map_err(|_| Error::Config("config-read"))?;
    check_meta(&meta)?;
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

#[cfg(unix)]
fn check_meta(meta: &std::fs::Metadata) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    if !meta.is_file() || (meta.permissions().mode() & 0o077) != 0 {
        return Err(Error::Config("config-mode"));
    }
    Ok(())
}
#[cfg(not(unix))]
fn check_meta(meta: &std::fs::Metadata) -> Result<()> {
    if !meta.is_file() {
        return Err(Error::Config("config-mode"));
    }
    Ok(())
}
