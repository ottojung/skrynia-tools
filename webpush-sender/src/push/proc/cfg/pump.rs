use super::field::{str_field, u32_field, u64_field, usize_field};
use super::hex::{hex16, hex32};
use super::{Config, Error, Result};
use serde_json::Value;

pub(super) fn parse_pump(v: &Value) -> Result<Config> {
    let key = str_field(v, "key")?.as_bytes().to_vec();
    let cfg = Config {
        sender_cap: hex32(v, "sender_cap")?,
        ns: hex32(v, "ns")?,
        key,
        max_payload: usize_field(v, "max_payload")?,
        max_attempts: u8::try_from(u64_field(v, "max_attempts")?)
            .map_err(|_| Error::Config("config-field"))?,
        base_backoff_ms: u64_field(v, "base_backoff_ms")?,
        max_backoff_ms: u64_field(v, "max_backoff_ms")?,
        ttl_secs: u32_field(v, "ttl_secs")?,
        topic: hex16(v, "topic")?,
        vapid_sub: str_field(v, "vapid_sub")?.to_owned(),
        ledger_epoch: u64_field(v, "ledger_epoch")?,
    };
    cfg.validate()?;
    Ok(cfg)
}
