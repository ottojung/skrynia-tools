use super::Store;
use anyhow::{Result, bail};

pub struct AppendLog<'a, S: Store> {
    pub(super) store: &'a S,
    prefix: String,
}
impl<'a, S: Store> AppendLog<'a, S> {
    pub fn open(store: &'a S, prefix: &str) -> Result<Self> {
        if prefix.is_empty()
            || prefix.starts_with('.')
            || !prefix
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || c == b'_' || c == b'.')
        {
            bail!("invalid log prefix");
        }
        Ok(Self {
            store,
            prefix: prefix.into(),
        })
    }
    pub(super) fn key(&self, n: u64) -> String {
        format!("{}.e000001.{n:012}", self.prefix)
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
