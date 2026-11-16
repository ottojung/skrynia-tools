use super::{CreateResult, EventSet, EventStore, Insert};
use anyhow::{Context, Result, bail};

impl<S: EventStore> EventSet<'_, S> {
    /// Identical retries are idempotent. An unknown write is reconciled by GET.
    pub fn insert(&self, id: &str, bytes: &[u8]) -> Result<Insert> {
        let key = self.key(id)?;
        match self.store.create(&key, bytes) {
            Ok(CreateResult::Created) => Ok(Insert::Created),
            Ok(CreateResult::Exists) => self.confirm(&key, bytes),
            Err(error) => match self.store.get(&key) {
                Ok(Some(current)) if current == bytes => Ok(Insert::AlreadyPresent),
                Ok(Some(_)) => bail!("event ID collision after uncertain write"),
                Ok(None) => Err(error.context("event creation outcome remains uncertain")),
                Err(check) => Err(error.context(format!(
                    "event creation outcome uncertain; reconciliation failed: {check}"
                ))),
            },
        }
    }
    fn confirm(&self, key: &str, bytes: &[u8]) -> Result<Insert> {
        let current = self
            .store
            .get(key)
            .context("verifying occupied event key")?;
        match current {
            Some(value) if value == bytes => Ok(Insert::AlreadyPresent),
            Some(_) => bail!("event ID collision with different bytes"),
            None => bail!("occupied event vanished; create-only assumption violated"),
        }
    }
}
