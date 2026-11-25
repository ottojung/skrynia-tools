//! Fail-injecting Disk implementation.
use super::first_support::CrashDisk;
use skrynia_native_service::cap::Disk;

impl Disk for CrashDisk {
    type Err = ();
    fn read_all(&self, p: &str) -> Result<Option<Vec<u8>>, ()> {
        Ok(self.files.borrow().get(p).cloned())
    }
    fn write_atomic(&self, p: &str, b: &[u8]) -> Result<(), ()> {
        let (before, after) = self.mutation();
        if before {
            return Err(());
        }
        self.files.borrow_mut().insert(p.into(), b.into());
        if after { Err(()) } else { Ok(()) }
    }
    fn remove(&self, p: &str) -> Result<(), ()> {
        let (before, after) = self.mutation();
        if before {
            return Err(());
        }
        self.files.borrow_mut().remove(p);
        if after { Err(()) } else { Ok(()) }
    }
    fn list(&self, path: &str) -> Result<Vec<String>, ()> {
        let prefix = format!("{}/", path.trim_end_matches('/'));
        Ok(self
            .files
            .borrow()
            .keys()
            .filter_map(|p| p.strip_prefix(&prefix))
            .filter_map(|p| p.split('/').next())
            .map(str::to_owned)
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect())
    }
}
