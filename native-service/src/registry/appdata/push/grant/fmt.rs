//! Durable grant record format. Body = namespace || key_len || key || subject.
//! The storage path also binds `grant_ns = SHA256(grant_cap)` and `SHA256(key)`.
use super::id;
use crate::hash_bytes;

pub enum Kind {
    Subscribe,
    Sender,
    Wake,
}

impl Kind {
    fn dir(&self) -> &'static str {
        match self {
            Kind::Subscribe => "pushgrant",
            Kind::Sender => "pushsender",
            Kind::Wake => "pushwake",
        }
    }
}

pub fn grant_path(base: &str, kind: &Kind, grant_ns: &[u8], key: &[u8]) -> String {
    format!(
        "{}/{}/{}/{}.bin",
        base.trim_end_matches('/'),
        kind.dir(),
        id::to_hex(grant_ns),
        id::to_hex(&hash_bytes(key))
    )
}
