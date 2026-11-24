//! Grant data is issuer namespace, exact key, exact subject, and revision.
use crate::CAP_LEN;

pub fn encode(ns: &[u8], key: &[u8], subject: &[u8], revision: u64) -> Vec<u8> {
    let mut out = ns.to_vec();
    out.push(key.len() as u8);
    out.extend_from_slice(key);
    out.extend_from_slice(subject);
    out.extend_from_slice(&revision.to_le_bytes());
    out
}

pub fn revision(bytes: &[u8], key: &[u8]) -> Option<u64> {
    let start = CAP_LEN + 1 + key.len();
    if bytes.len() != start + CAP_LEN + 8
        || bytes[CAP_LEN] as usize != key.len()
        || &bytes[CAP_LEN + 1..start] != key
    {
        return None;
    }
    Some(u64::from_le_bytes(
        bytes[start + CAP_LEN..].try_into().ok()?,
    ))
}

pub fn decode(bytes: &[u8], key: &[u8], subject: &[u8]) -> Option<[u8; CAP_LEN]> {
    revision(bytes, key)?;
    let start = CAP_LEN + 1 + key.len();
    if subject.len() != CAP_LEN || &bytes[start..start + CAP_LEN] != subject {
        return None;
    }
    bytes[..CAP_LEN].try_into().ok()
}

pub fn owned_by(bytes: &[u8], ns: &[u8], key: &[u8]) -> bool {
    revision(bytes, key).is_some() && &bytes[..CAP_LEN] == ns
}
