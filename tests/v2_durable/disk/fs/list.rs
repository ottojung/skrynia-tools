use super::Fs;
use std::{fs, io};

pub fn list(fs: &Fs, d: &str) -> Result<Vec<String>, io::Error> {
    let mut out = Vec::new();
    let rd = match fs::read_dir(fs.path(d)) {
        Ok(r) => r,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(out),
        Err(e) => return Err(e),
    };
    for e in rd {
        out.push(e?.file_name().to_string_lossy().into_owned());
    }
    out.sort();
    Ok(out)
}

pub fn ensure(fs: &Fs, d: &str) -> Result<(), io::Error> {
    fs::create_dir_all(fs.path(d))
}
