//! Versioned subscription record binds an exact grant generation and subject.
use super::{CAP_LEN, MAX_SUB_BYTES};
const V1: u8 = 1;
const HEADER: usize = 1 + CAP_LEN + 8 + CAP_LEN;

pub fn encode(
    cap_hash: &[u8; CAP_LEN],
    generation: u64,
    subject_hash: &[u8; CAP_LEN],
    endpoint: &[u8],
) -> Vec<u8> {
    let mut out = Vec::with_capacity(HEADER + endpoint.len());
    out.push(V1);
    out.extend_from_slice(cap_hash);
    out.extend_from_slice(&generation.to_le_bytes());
    out.extend_from_slice(subject_hash);
    out.extend_from_slice(endpoint);
    out
}

pub struct Parts<'a> {
    pub cap_hash: &'a [u8],
    pub generation: u64,
    pub subject_hash: &'a [u8],
    pub endpoint: &'a [u8],
}

pub fn parts(bytes: &[u8]) -> Option<Parts<'_>> {
    if bytes.len() <= HEADER || bytes.len() > HEADER + MAX_SUB_BYTES || bytes[0] != V1 {
        return None;
    }
    let cap_hash = &bytes[1..1 + CAP_LEN];
    let generation = u64::from_le_bytes(bytes[1 + CAP_LEN..1 + CAP_LEN + 8].try_into().ok()?);
    let subject_hash = &bytes[1 + CAP_LEN + 8..HEADER];
    Some(Parts {
        cap_hash,
        generation,
        subject_hash,
        endpoint: &bytes[HEADER..],
    })
}
