use super::error::{Error, Result};

pub const CAP_LEN: usize = 32;
pub const MAX_KEY: usize = 255;
pub const MAX_PAYLOAD: usize = 4096;
pub const MAX_HINTS: usize = 1024;
pub const MAX_SUBS: usize = 64;
pub const TOPIC_LEN: usize = 16;

/// All identity material is injected. No token or VAPID key is a constant.
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
}

impl Config {
    pub fn validate(&self) -> Result<()> {
        if self.key.is_empty() || self.key.len() > MAX_KEY {
            return Err(Error::Config("key"));
        }
        if self.key.iter().any(|b| *b == 0 || *b < 0x20) {
            return Err(Error::Config("key-bytes"));
        }
        if self.max_payload == 0 || self.max_payload > MAX_PAYLOAD {
            return Err(Error::Config("payload"));
        }
        if self.max_attempts == 0 || self.max_attempts > 5 {
            return Err(Error::Config("attempts"));
        }
        // RFC 8292 §2: VAPID `exp` may be at most 24 hours in the future.
        if self.ttl_secs == 0 || self.ttl_secs > 86_400 {
            return Err(Error::Config("ttl"));
        }
        // Reject JSON metacharacters and controls: claims are serialized, but a
        // malformed contact address must fail closed instead of being signed.
        if !self.vapid_sub.starts_with("mailto:")
            || self.vapid_sub.len() > 254
            || self
                .vapid_sub
                .bytes()
                .any(|b| b < 0x20 || b == 0x7f || b == b'"' || b == b'\\')
        {
            return Err(Error::Config("sub"));
        }
        Ok(())
    }
}
