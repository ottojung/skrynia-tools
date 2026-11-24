//! Byte image of the bounded ledger: header, records, keyed MAC trailer.
use super::format::{HEAD, MAGIC, TRAILER};
use super::model::{Record, tag};
#[cfg(test)]
use super::{Decision, DurableLedger, FileLedgerStore, model};

pub fn encode(binding: &[u8; 32], cap: usize, records: &[Record], key: &[u8; 32]) -> Vec<u8> {
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
    out.extend_from_slice(&tag(&out, key));
    out
}

#[cfg(test)]
mod test {
    use super::{Decision, DurableLedger, FileLedgerStore, model};
    use sha2::Digest;

    #[test]
    fn file_store_roundtrip_restart_and_forged_tag_rejected() {
        let cfg = model::test_cfg();
        let dir = std::env::temp_dir().join(format!("skledger-{}-1", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let store = FileLedgerStore::from_config(&dir, &cfg).unwrap();
        let mut a = DurableLedger::open(store, &cfg).unwrap();
        a.mark_delivered(7).unwrap();
        a.mark_acked(7).unwrap();
        let again = FileLedgerStore::from_config(&dir, &cfg).unwrap();
        let b = DurableLedger::open(again, &cfg).unwrap();
        assert_eq!(b.decide(7), Ok(Decision::Acked));
        // A directory writer can recompute the OLD unkeyed checksum, but not
        // the keyed tag: this forged image must still fail closed.
        let mut entries = std::fs::read_dir(&dir).unwrap();
        let path = entries.next().unwrap().unwrap().path();
        let mut bytes = std::fs::read(&path).unwrap();
        let split = bytes.len() - 32;
        bytes[super::HEAD] ^= 1;
        let mut h = sha2::Sha256::new();
        h.update(b"skrynia-ledger-digest-v1");
        h.update(&bytes[..split]);
        bytes[split..].copy_from_slice(&h.finalize()[..]);
        std::fs::write(&path, bytes).unwrap();
        let forged = FileLedgerStore::from_config(&dir, &cfg).unwrap();
        assert!(DurableLedger::open(forged, &cfg).is_err());
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
