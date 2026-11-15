use super::{CreateResult, Store, entry, types::AppendLog};
use anyhow::{Result, bail};

impl<'a, S: Store> AppendLog<'a, S> {
    pub fn append(&self, operation_id: &str, data: &[u8]) -> Result<u64> {
        let bytes = entry::encode(operation_id, data)?;
        if self.store.get(&self.key(0))?.as_deref() != Some(b"skrynia-log-genesis-v1") {
            bail!("log must be initialized before appending");
        }
        for n in 1..=1_000_000_u64 {
            let key = self.key(n);
            if let Some(previous) = self.store.get(&key)? {
                if entry::observed(&previous, &bytes, operation_id)? {
                    return Ok(n);
                }
                continue;
            }
            match self.store.create(&key, &bytes) {
                Ok(CreateResult::Created) => return Ok(n),
                Ok(CreateResult::Exists) => continue,
                Err(error) => match self.store.get(&key) {
                    Ok(Some(current)) if current == bytes => return Ok(n),
                    Ok(Some(_)) => continue,
                    Ok(None) => {
                        return Err(error.context("append outcome uncertain; slot still absent"));
                    }
                    Err(check) => {
                        return Err(error.context(format!(
                            "append outcome uncertain; reconciliation failed: {check}"
                        )));
                    }
                },
            }
        }
        bail!("log exceeds bounded one-million-slot scan")
    }
}
