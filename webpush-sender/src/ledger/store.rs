//! Production store: operator-supplied directory, grant-hash filename,
//! atomic durable replace. No shared, hardcoded or unsecured path.
use super::{Error, LedgerStore, Result};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

pub struct FileLedgerStore {
    final_path: PathBuf,
    tmp_path: PathBuf,
}

impl FileLedgerStore {
    /// Production entry point: derive the binding from the exact `Config` so
    /// the filename and the in-file header cannot diverge. `dir` must be a
    /// private (0700), single-writer operator capability; it is never shared.
    pub fn from_config(dir: &Path, cfg: &crate::Config) -> Result<Self> {
        Self::new(dir, &super::ledger_binding(cfg))
    }

    /// `dir` is an injected capability. The file name is a one-way grant
    /// binding, never raw cap/ns/key; separate grants never share a file.
    pub fn new(dir: &Path, binding: &[u8; 32]) -> Result<Self> {
        if dir.as_os_str().is_empty() {
            return Err(Error::Config("ledger-dir"));
        }
        let name: String = binding.iter().map(|b| format!("{b:02x}")).collect();
        let mut final_path = dir.to_path_buf();
        final_path.push(format!("ledger-{name}.bin"));
        let mut tmp_path = final_path.clone();
        tmp_path.set_extension("tmp");
        Ok(Self {
            final_path,
            tmp_path,
        })
    }
}

impl LedgerStore for FileLedgerStore {
    fn load(&self) -> Result<Option<Vec<u8>>> {
        let mut file = match File::open(&self.final_path) {
            Ok(file) => file,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(_) => return Err(Error::Transport("ledger-read")),
        };
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)
            .map_err(|_| Error::Transport("ledger-read"))?;
        Ok(Some(bytes))
    }

    fn save(&self, bytes: &[u8]) -> Result<()> {
        super::commit::replace(&self.final_path, &self.tmp_path, bytes)
    }
}
