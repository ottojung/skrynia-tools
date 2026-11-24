//! Sparse keys, long shared prefixes and arbitrary/stale cursors.
use super::support::crafted;
use crate::cases::support::MemDisk;
use skrynia_native_service::index::{self, Scope};

#[test]
fn adversarial_shared_prefix_sparse_keys_and_arbitrary_cursors() {
    let d = MemDisk::default();
    let scope = Scope { base: "reg" };
    let prefix = [0xabu8; 16];
    let mut keys = Vec::new();
    for i in 0..300u16 {
        let k = crafted(&prefix, i * 3 + 1);
        index::insert(&d, scope, k).unwrap();
        keys.push(k);
    }
    keys.sort();
    let mut seen = Vec::new();
    let mut after = None;
    loop {
        let p = index::page(&d, scope, &after, 64).unwrap();
        if p.is_empty() {
            break;
        }
        seen.extend_from_slice(&p);
        if p.len() < 64 {
            break;
        }
        after = Some(*p.last().unwrap());
    }
    assert_eq!(seen, keys);
    let mid = keys[150];
    assert_eq!(
        index::page(&d, scope, &Some(mid), 7).unwrap(),
        keys[151..158]
    );
    let stale = crafted(&prefix, 150 * 3 + 2);
    let p2 = index::page(&d, scope, &Some(stale), 3).unwrap();
    assert!(!p2.is_empty() && p2.iter().all(|k| k > &stale));
}
