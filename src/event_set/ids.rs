use super::{EventSet, EventStore};
use anyhow::{Result, bail};
impl<S: EventStore> EventSet<'_, S> {
    pub(super) fn key(&self, id: &str) -> Result<String> {
        if id.len() != 32
            || !id
                .bytes()
                .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
        {
            bail!("event ID must be 32 lowercase hexadecimal characters");
        }
        Ok(format!("{}.{}", self.prefix, id))
    }
}
