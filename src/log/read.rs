use super::{Store, entry, types::AppendLog};
use anyhow::Result;

impl<'a, S: Store> AppendLog<'a, S> {
    pub fn read_from(&self, start: u64, limit: usize) -> Result<Vec<(u64, String, Vec<u8>)>> {
        let mut out = Vec::new();
        for n in start.max(1)..start.max(1).saturating_add(limit as u64) {
            let Some(bytes) = self.store.get(&self.key(n))? else {
                break;
            };
            let (id, payload) = entry::decode(&bytes)?;
            out.push((n, id.to_owned(), payload.to_vec()));
        }
        Ok(out)
    }
}
