//! Byte image of the bounded ledger: header, records, SHA-256 trailer.
use super::format::{HEAD, MAGIC, TRAILER};
use super::model::{Record, digest};

pub fn encode(binding: &[u8; 32], cap: usize, records: &[Record]) -> Vec<u8> {
    let mut out = Vec::with_capacity(HEAD + 16 * records.len() + TRAILER);
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(binding);
    out.extend_from_slice(&(cap as u32).to_le_bytes());
    out.extend_from_slice(&(records.len() as u32).to_le_bytes());
    for r in records {
        out.extend_from_slice(&r.seq.to_le_bytes());
        out.push(r.state as u8);
        out.extend_from_slice(&[0u8; 7]);
    }
    out.extend_from_slice(&digest(&out));
    out
}
