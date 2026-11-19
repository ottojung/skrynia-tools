use super::Fs;
use std::{fs, io};

pub fn read_all(fs: &Fs, p: &str) -> Result<Option<Vec<u8>>, io::Error> {
    match fs::read(fs.path(p)) {
        Ok(b) => Ok(Some(b)),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e),
    }
}

pub fn write_atomic(fs: &Fs, p: &str, b: &[u8]) -> Result<(), io::Error> {
    let full = fs.path(p);
    if let Some(parent) = full.parent() {
        fs::create_dir_all(parent)?;
    }
    let tmp = full.with_extension("tmp");
    fs::write(&tmp, b)?;
    std::fs::rename(tmp, full)
}

pub fn remove(fs: &Fs, p: &str) -> Result<(), io::Error> {
    match std::fs::remove_file(fs.path(p)) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e),
    }
}
