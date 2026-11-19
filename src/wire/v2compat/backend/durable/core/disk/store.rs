use skrynia_program0::cap::Disk;
use std::fmt;

const DIR: &str = "v2durable";
const STATE: &str = "v2durable/state.json";
const UNCERTAIN: &str = "v2durable/uncertain";

pub struct DiskStateStore<D> {
    disk: D,
}

impl<D> DiskStateStore<D> {
    pub fn new(disk: D) -> Self {
        Self { disk }
    }
}

#[derive(Debug)]
pub struct DiskStoreError(String);

impl fmt::Display for DiskStoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

fn map_err<E: fmt::Display>(e: E) -> DiskStoreError {
    DiskStoreError(e.to_string())
}

impl<D: Disk> super::super::super::data::StateStore for DiskStateStore<D>
where
    D::Err: fmt::Display,
{
    type Error = DiskStoreError;
    fn load(&self) -> Result<Option<Vec<u8>>, Self::Error> {
        self.disk.ensure_directory(DIR).map_err(map_err)?;
        if self.disk.read_all(UNCERTAIN).map_err(map_err)?.is_some() {
            return Err(DiskStoreError("state_uncertain".into()));
        }
        self.disk.read_all(STATE).map_err(map_err)
    }
    fn save(&self, bytes: &[u8]) -> Result<(), Self::Error> {
        self.disk.ensure_directory(DIR).map_err(map_err)?;
        self.disk.write_atomic(UNCERTAIN, b"1").map_err(map_err)?;
        self.disk.write_atomic(STATE, bytes).map_err(map_err)?;
        self.disk.remove(UNCERTAIN).map_err(map_err)
    }
}
