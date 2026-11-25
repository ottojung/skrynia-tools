use super::{Error, Result};
use std::fs::{File, OpenOptions};
#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};

pub struct Lock {
    _file: File,
    _path: PathBuf,
}

impl Lock {
    pub fn acquire(dir: &Path) -> Result<Self> {
        let path = dir.join("sender.lock");
        let mut opts = OpenOptions::new();
        opts.read(true).write(true).create(true).truncate(false);
        #[cfg(unix)]
        opts.mode(0o600);
        let file = opts.open(&path).map_err(|_| Error::Config("lock-open"))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            file.set_permissions(std::fs::Permissions::from_mode(0o600))
                .map_err(|_| Error::Config("lock-open"))?;
        }
        file.try_lock().map_err(|_| Error::Config("lock-held"))?;
        Ok(Self {
            _file: file,
            _path: path,
        })
    }
}
