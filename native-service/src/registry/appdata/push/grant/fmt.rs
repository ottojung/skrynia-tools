//! Durable grant record format. Body = namespace || key_len || key || subject.
//! The storage path also binds `grant_ns = SHA256(grant_cap)` and `SHA256(key)`.
use super::id;
use crate::{CAP_LEN, hash_bytes};

pub enum Kind {
    Subscribe,
    Sender,
}

impl Kind {
    fn dir(&self) -> &'static str {
        match self {
            Kind::Subscribe => "pushgrant",
            Kind::Sender => "pushsender",
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

pub fn encode(ns: &[u8], key: &[u8], subject: &[u8]) -> Vec<u8> {
    let mut out = ns.to_vec();
    out.push(key.len() as u8);
    out.extend_from_slice(key);
    out.extend_from_slice(subject);
    out
}

/// Recover the granted namespace only if the exact key and subject match.
pub fn decode(bytes: &[u8], key: &[u8], subject: &[u8]) -> Option<[u8; CAP_LEN]> {
    if bytes.len() < CAP_LEN + 2 || bytes[CAP_LEN] as usize != key.len() {
        return None;
    }
    let end = CAP_LEN + 1 + key.len();
    if bytes.len() != end + subject.len()
        || &bytes[CAP_LEN + 1..end] != key
        || &bytes[end..] != subject
    {
        return None;
    }
    bytes[..CAP_LEN].try_into().ok()
}

/// Existing grant may only be updated by its original namespace writer.
pub fn owned_by(bytes: &[u8], ns: &[u8], key: &[u8]) -> bool {
    let end = CAP_LEN + 1 + key.len();
    bytes.len() == end + CAP_LEN
        && bytes[CAP_LEN] as usize == key.len()
        && &bytes[..CAP_LEN] == ns
        && &bytes[CAP_LEN + 1..end] == key
}
