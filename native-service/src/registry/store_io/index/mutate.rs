//! Immutable-node entry codecs and sorted insertion helpers.
use super::{KEY_LEN, find};

pub type LeafEntry = ([u8; KEY_LEN], bool);
pub type DirEntry = ([u8; KEY_LEN], u32);

pub fn leaves(body: &[u8]) -> Vec<LeafEntry> {
    let mut out = Vec::new();
    for i in 0..body.len() / (KEY_LEN + 1) {
        let (Some(k), true) = (
            find::leaf_key(body, i),
            find::leaf_flag(body, i).unwrap_or(false),
        ) else {
            continue;
        };
        let mut key = [0u8; KEY_LEN];
        key.copy_from_slice(k);
        out.push((key, find::leaf_flag(body, i).unwrap_or(false)));
    }
    out
}

pub fn dirs(body: &[u8]) -> Vec<DirEntry> {
    let mut out = Vec::new();
    for i in 0..body.len() / (KEY_LEN + 4) {
        let (Some(k), Some(id @ 1..)) = (find::dir_key(body, i), find::dir_leaf(body, i)) else {
            continue;
        };
        let mut key = [0u8; KEY_LEN];
        key.copy_from_slice(k);
        out.push((key, id));
    }
    out
}

pub fn put_leaf(es: &mut Vec<LeafEntry>, key: [u8; KEY_LEN], live: bool) {
    match es.binary_search_by(|e| e.0.cmp(&key)) {
        Ok(i) => es[i].1 = live,
        Err(i) => es.insert(i, (key, live)),
    }
}

pub fn remove_leaf(es: &mut Vec<LeafEntry>, key: &[u8; KEY_LEN]) -> bool {
    match es.binary_search_by(|e| e.0.cmp(key)) {
        Ok(i) => {
            es.remove(i);
            true
        }
        Err(_) => false,
    }
}
