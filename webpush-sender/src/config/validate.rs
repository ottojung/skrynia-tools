use super::{Config, Error, MAX_KEY, MAX_PAYLOAD, Result};

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
        // Bounded so a malicious config cannot request an unbounded sleep.
        if self.base_backoff_ms == 0
            || self.max_backoff_ms < self.base_backoff_ms
            || self.max_backoff_ms > 300_000
        {
            return Err(Error::Config("backoff"));
        }
        // RFC 8292 §2: VAPID `exp` may be at most 24 hours in the future.
        if self.ttl_secs == 0 || self.ttl_secs > 86_400 {
            return Err(Error::Config("ttl"));
        }
        let sub = &self.vapid_sub;
        if !sub.starts_with("mailto:")
            || sub.len() > 254
            || sub
                .bytes()
                .any(|b| b < 0x20 || b == 0x7f || b == b'"' || b == b'\\')
        {
            return Err(Error::Config("sub"));
        }
        Ok(())
    }
}
