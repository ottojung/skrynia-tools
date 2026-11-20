use std::{fs::OpenOptions, path::Path};
pub fn load(path: &Path) -> Result<[u8; 32], ()> {
    #[cfg(unix)]
    use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    options.custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC);
    let file = options.open(path).map_err(|_| ())?;
    let metadata = file.metadata().map_err(|_| ())?;
    if !metadata.file_type().is_file() {
        return Err(());
    }
    #[cfg(unix)]
    if metadata.permissions().mode() & 0o077 != 0 {
        return Err(());
    }
    use std::io::Read;
    let mut bytes = Vec::with_capacity(33);
    file.take(33).read_to_end(&mut bytes).map_err(|_| ())?;
    bytes.try_into().map_err(|_| ())
}
