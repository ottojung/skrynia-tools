mod io;
mod list;

use skrynia_program0::cap::Disk;
use std::{io as stdio, path::PathBuf};

#[derive(Clone)]
pub struct Fs(pub PathBuf);

impl Fs {
    pub fn new(dir: PathBuf) -> Self {
        std::fs::create_dir_all(&dir).unwrap();
        Self(dir)
    }
    fn path(&self, p: &str) -> PathBuf {
        self.0.join(p)
    }
}

impl Disk for Fs {
    type Err = stdio::Error;
    fn read_all(&self, p: &str) -> Result<Option<Vec<u8>>, stdio::Error> {
        io::read_all(self, p)
    }
    fn write_atomic(&self, p: &str, b: &[u8]) -> Result<(), stdio::Error> {
        io::write_atomic(self, p, b)
    }
    fn remove(&self, p: &str) -> Result<(), stdio::Error> {
        io::remove(self, p)
    }
    fn list(&self, d: &str) -> Result<Vec<String>, stdio::Error> {
        list::list(self, d)
    }
    fn ensure_directory(&self, d: &str) -> Result<(), stdio::Error> {
        list::ensure(self, d)
    }
}
