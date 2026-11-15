use super::{Store, wire};
use anyhow::{Result, bail};

pub struct AppendLog<'a, S: Store> {
    pub(super) store: &'a S,
    prefix: String,
}
impl<'a, S: Store> AppendLog<'a, S> {
    pub fn open(store: &'a S, prefix: &str) -> Result<Self> {
        if !wire::valid_prefix(prefix) {
            bail!("invalid log prefix");
        }
        Ok(Self {
            store,
            prefix: prefix.into(),
        })
    }
    pub(super) fn key(&self, n: u64) -> String {
        wire::key(&self.prefix, n)
    }
    pub fn initialize(&self) -> Result<()> {
        use super::CreateResult;
        match self.store.create(&self.key(0), b"skrynia-log-genesis-v1")? {
            CreateResult::Created => Ok(()),
            CreateResult::Exists => {
                if self.store.get(&self.key(0))?.as_deref() == Some(b"skrynia-log-genesis-v1") {
                    Ok(())
                } else {
                    bail!("foreign genesis occupies log prefix")
                }
            }
        }
    }
}
