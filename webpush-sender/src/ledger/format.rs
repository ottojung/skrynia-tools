//! Length-checked, checksummed decode. Any malformed or foreign file is
//! rejected before the ledger is used (fail closed, never treated as empty).
use super::model::{Record, State, tag};
use super::{Error, Result};

pub const MAGIC: &[u8; 8] = b"SKRLGRV1";
pub const HEAD: usize = 48;
pub const TRAILER: usize = 32;

pub fn decode(src: &[u8], binding: &[u8; 32], cap: usize, key: &[u8; 32]) -> Result<Vec<Record>> {
    if src.len() < HEAD + TRAILER {
        return Err(Error::Wire("ledger-short"));
    }
    let (body, tail) = src.split_at(src.len() - TRAILER);
    // Cheap public-field checks first so a foreign grant still reports the
    // precise Config("ledger-binding") error; the keyed tag is the integrity
    // barrier that an attacker with directory write alone cannot recompute.
    if &body[..8] != MAGIC {
        return Err(Error::Wire("ledger-magic"));
    }
    if &body[8..40] != binding {
        return Err(Error::Config("ledger-binding"));
    }
    if tag(body, key) != tail {
        return Err(Error::Wire("ledger-digest"));
    }
    let want = u32::from_le_bytes(body[40..44].try_into().expect("4")) as usize;
    let count = u32::from_le_bytes(body[44..48].try_into().expect("4")) as usize;
    if want != cap || count > cap || body.len() != HEAD + 16 * count {
        return Err(Error::Wire("ledger-shape"));
    }
    records(body)
}

fn records(body: &[u8]) -> Result<Vec<Record>> {
    let mut out: Vec<Record> = Vec::new();
    for chunk in body[HEAD..].chunks_exact(16) {
        let seq = u64::from_le_bytes(chunk[..8].try_into().expect("8"));
        let state = match chunk[8] {
            1 => State::Delivered,
            2 => State::Acked,
            _ => return Err(Error::Wire("ledger-state")),
        };
        if chunk[9..].iter().any(|b| *b != 0) || out.iter().any(|r| r.seq == seq) {
            return Err(Error::Wire("ledger-record"));
        }
        out.push(Record { seq, state });
    }
    Ok(out)
}
