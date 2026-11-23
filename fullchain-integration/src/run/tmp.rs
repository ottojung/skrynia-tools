//! Prepare an exclusive private directory without deleting any existing path.
use std::{
    fs, io,
    path::{Path, PathBuf},
};

pub fn create(requested: Option<PathBuf>) -> io::Result<PathBuf> {
    if let Some(path) = requested {
        private_dir(&path)?;
        return Ok(path);
    }
    let base = std::env::temp_dir();
    for attempt in 0..64 {
        let path = base.join(format!("fc-webpush-{}-{attempt}", std::process::id()));
        match private_dir(&path) {
            Ok(()) => return Ok(path),
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {}
            Err(e) => return Err(e),
        }
    }
    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "no fresh test directory",
    ))
}

fn private_dir(path: &Path) -> io::Result<()> {
    let mut builder = fs::DirBuilder::new();
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(0o700);
    }
    builder.create(path)
}

#[cfg(test)]
mod tests;

#[cfg(all(test, unix))]
mod symlink_test;
