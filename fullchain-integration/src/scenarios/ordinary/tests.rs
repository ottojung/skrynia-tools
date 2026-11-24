//! Exercise the 257th write through real WASM; no O(N) callback scan allowed.
use super::{ctx::Ctx, wire as w};
use crate::{blb, idb, must, req};
pub fn fill_and_check(c: &Ctx) {
    let mut old = None;
    for i in 0..256u16 {
        let value = i.to_le_bytes();
        let cas = old.map_or(vec![0], |h: [u8; 32]| {
            let mut v = vec![1]; v.extend_from_slice(&h); v
        });
        must(&c.io, &req(6, &[
            &idb(&w::writer()), &blb(w::KEY), &cas, &blb(&value),
        ]), "ordinary put");
        old = Some(w::h(&value));
    }
    let initial = w::list(c);
    assert_eq!(initial.len(), 256, "256 legacy hash hints survive");
    for (i, seq) in initial.iter().enumerate() {
        assert_eq!(w::read(c, *seq), w::h(&(i as u16).to_le_bytes()));
    }
    let mut cas = vec![1]; cas.extend_from_slice(&old.unwrap());
    must(&c.io, &req(6, &[
        &idb(&w::writer()), &blb(w::KEY), &cas, &blb(b"late mutation"),
    ]), "late put");
    let now = w::list(c);
    assert_eq!(now.len(), 1, "one overflow generation");
    assert_ne!(now[0] & (1u64 << 63), 0, "high-bit fallback seq");
    assert_eq!(w::read(c, now[0]), [0u8; 32]);
}
