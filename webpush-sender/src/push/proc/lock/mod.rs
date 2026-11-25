mod check;
use super::{Error, Result, self_uid};
use std::fs::{File, OpenOptions};
#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};

pub struct Lock {
    _file: File,
    _path: PathBuf,
}

impl Lock {
    /// Acquire a persistent single-writer inode. Never delete, truncate or
    /// chmod an existing lock: a stale lock is an operator decision.
    pub fn acquire(dir: &Path) -> Result<Self> {
        let path = dir.join("sender.lock");
        let file = match create(&path)? {
            Some(f) => f,
            None => open(&path)?,
        };
        check::check(&file)?;
        file.try_lock().map_err(|_| Error::Config("lock-held"))?;
        Ok(Self {
            _file: file,
            _path: path,
        })
    }
}

fn create(path: &Path) -> Result<Option<File>> {
    let mut opts = OpenOptions::new();
    opts.write(true).create_new(true);
    #[cfg(unix)]
    {
        opts.mode(0o600);
        opts.custom_flags(libc::O_CLOEXEC | libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    match opts.open(path) {
        Ok(f) => Ok(Some(f)),
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => Ok(None),
        Err(_) => Err(Error::Config("lock-open")),
    }
}

fn open(path: &Path) -> Result<File> {
    let mut opts = OpenOptions::new();
    opts.read(true).write(true);
    #[cfg(unix)]
    {
        opts.custom_flags(libc::O_CLOEXEC | libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    opts.open(path).map_err(|_| Error::Config("lock-open"))
}
