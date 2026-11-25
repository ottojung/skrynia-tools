//! All identity material is injected. No token or VAPID key is a constant.
mod validate;
pub use super::error::{Error, Result};

pub const CAP_LEN: usize = 32;
pub const MAX_KEY: usize = 255;
pub const MAX_PAYLOAD: usize = 4096;
pub const MAX_HINTS: usize = 1024;
pub const MAX_SUBS: usize = 64;
pub const TOPIC_LEN: usize = 16;

#[derive(Clone)]
pub struct Config {
    pub sender_cap: [u8; CAP_LEN],
    pub ns: [u8; CAP_LEN],
    pub key: Vec<u8>,
    pub max_payload: usize,
    pub max_attempts: u8,
    pub base_backoff_ms: u64,
    pub max_backoff_ms: u64,
    pub ttl_secs: u32,
    pub topic: [u8; TOPIC_LEN],
    pub vapid_sub: String,
    /// Bump when P1 hint highwater is reset/migrated: the ledger is re-bound.
    pub ledger_epoch: u64,
}
