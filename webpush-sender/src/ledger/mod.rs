//! Bounded durable replay ledger bound to one EXACT P1 sender grant.
//! Lives in the sender: P1 stays generic and must gain no WebPush state.
mod commit;
mod fake;
mod format;
mod model;
mod store;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests2;
mod update;
mod wire;

pub use fake::{Faults, MemoryLedgerStore};
pub use model::{Record, State};
pub use store::FileLedgerStore;

use super::Config;
use super::config::MAX_HINTS;
use super::error::{Error, Result};
use format::decode;
use model::binding;
use wire::encode;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Decision {
    Fresh,
    Delivered,
    Acked,
}

/// Injectable durable byte store. `save` must be atomic and fsynced before it
/// returns, so a persisted marker is a real durability boundary.
pub trait LedgerStore {
    fn load(&self) -> Result<Option<Vec<u8>>>;
    fn save(&self, bytes: &[u8]) -> Result<()>;
}

pub struct DurableLedger {
    pub(super) store: Box<dyn LedgerStore>,
    pub(super) binding: [u8; 32],
    pub(super) records: Vec<Record>,
}

impl DurableLedger {
    /// Load and validate before any delivery. Corruption or a foreign grant
    /// binding fails closed; an absent file starts an empty bounded ledger.
    pub fn open<S: LedgerStore + 'static>(store: S, cfg: &Config) -> Result<Self> {
        let binding = binding(&cfg.sender_cap, &cfg.ns, &cfg.key, cfg.ledger_epoch);
        let records = match store.load()? {
            None => Vec::new(),
            Some(bytes) => decode(&bytes, &binding, MAX_HINTS)?,
        };
        Ok(Self {
            store: Box::new(store),
            binding,
            records,
        })
    }
}
