//! Crash-safe replace: write temp, fsync file, rename, fsync parent directory.
//! After rename, a dir-fsync failure means rename durability is uncertain.
use super::{Error, Result};
use std::fs::{File, OpenOptions};
use std::io::Write;
#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;

pub fn replace(final_path: &Path, tmp_path: &Path, bytes: &[u8]) -> Result<()> {
    let mut file = open_tmp(tmp_path)?;
    file.write_all(bytes)
        .and_then(|()| file.sync_all())
        .map_err(|e| classify(e, "ledger-sync"))?;
    drop(file);
    std::fs::rename(tmp_path, final_path).map_err(|_| Error::Transport("ledger-rename"))?;
    let dir =
        File::open(final_path.parent().unwrap()).map_err(|_| Error::Transport("ledger-dir"))?;
    dir.sync_all().map_err(|e| classify(e, "ledger-dir-sync"))
}

fn open_tmp(path: &Path) -> Result<File> {
    let mut opts = OpenOptions::new();
    opts.write(true).create_new(true);
    #[cfg(unix)]
    opts.mode(0o600)
        .custom_flags(libc::O_CLOEXEC | libc::O_NOFOLLOW);
    match opts.open(path) {
        Ok(f) => Ok(f),
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            let _ = std::fs::remove_file(path);
            opts.open(path).map_err(|e| classify(e, "ledger-open"))
        }
        Err(e) => Err(classify(e, "ledger-open")),
    }
}

fn classify(e: std::io::Error, label: &'static str) -> Error {
    if e.kind() == std::io::ErrorKind::StorageFull || e.raw_os_error() == Some(28) {
        Error::Quota
    } else {
        Error::Transport(label)
    }
}
