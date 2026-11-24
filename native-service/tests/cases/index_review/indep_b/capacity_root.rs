//! Full-root capacity rejection before any durable mutation.
use super::capacity_root_help::{leaf, seal, set_dir};
use super::legacy_support::Mem as Memory;
pub fn key(n: u32) -> [u8; 32] {
    let mut b = [0u8; 32];
    b[28..].copy_from_slice(&n.to_be_bytes());
    b
}
use skrynia_native_service::cap::Disk;
use skrynia_native_service::index::{self, Scope};

#[test]
fn full_root_split_rejects_before_any_mutation() {
    let d = Memory::default();
    let s = Scope { base: "reg" };
    let mut root = vec![0u8; 256 * 36];
    let mut nl = 0u32;
    for di in 0..256u32 {
        let start = di * 1_000_000 + 1;
        let mut dir = vec![0u8; 128 * 36];
        for li in 0..if di == 0 { 128 } else { 1 } {
            nl += 1;
            let k = start + li * 1000;
            set_dir(&mut dir, li as usize, key(k), nl);
            d.write_atomic(
                &s.leaf(nl),
                &leaf(k, if di == 0 && li == 0 { 124 } else { 1 }),
            )
            .unwrap();
        }
        d.write_atomic(&s.dir(di + 1), &seal(dir)).unwrap();
        set_dir(&mut root, di as usize, key(start), di + 1);
    }
    d.write_atomic(&s.root(1), &seal(root)).unwrap();
    let mut meta = b"IX01".to_vec();
    meta.extend_from_slice(&256u32.to_le_bytes());
    meta.extend_from_slice(&nl.to_le_bytes());
    meta.extend_from_slice(&1u32.to_le_bytes());
    d.write_atomic(&s.meta(), &seal(meta)).unwrap();
    let before = d.0.borrow().clone();
    let r = index::insert(&d, s, key(125));
    assert!(r.is_err(), "over-capacity root split must return an error");
    assert_eq!(
        *d.0.borrow(),
        before,
        "rejected split changed persistent state"
    );
}
