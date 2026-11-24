//! Encode sorted entry vectors into fixed-width node bodies.
use super::mutate::{DirEntry, LeafEntry};
use super::{DIR_CAP, KEY_LEN, LEAF_CAP, ROOT_CAP};

pub fn leaf(es: &[LeafEntry]) -> Vec<u8> {
    let mut b = vec![0u8; LEAF_CAP * (KEY_LEN + 1)];
    for (i, (k, live)) in es.iter().enumerate() {
        b[i * (KEY_LEN + 1)..i * (KEY_LEN + 1) + KEY_LEN].copy_from_slice(k);
        b[i * (KEY_LEN + 1) + KEY_LEN] = u8::from(*live);
    }
    b
}

fn nodes(es: &[DirEntry], cap: usize) -> Vec<u8> {
    let mut b = vec![0u8; cap * (KEY_LEN + 4)];
    for (i, (k, id)) in es.iter().enumerate() {
        b[i * (KEY_LEN + 4)..i * (KEY_LEN + 4) + KEY_LEN].copy_from_slice(k);
        let off = i * (KEY_LEN + 4) + KEY_LEN;
        b[off..off + 4].copy_from_slice(&id.to_le_bytes());
    }
    b
}

pub fn dir(es: &[DirEntry]) -> Vec<u8> {
    nodes(es, DIR_CAP)
}

pub fn root(es: &[DirEntry]) -> Vec<u8> {
    nodes(es, ROOT_CAP)
}
