//! Entry access, binary search and empty-node builders for index nodes.
use super::*;

pub fn dir_key(body: &[u8], i: usize) -> Option<&[u8]> {
    body.get(i * (KEY_LEN + 4)..i * (KEY_LEN + 4) + KEY_LEN)
}
pub fn dir_leaf(body: &[u8], i: usize) -> Option<u32> {
    let off = i * (KEY_LEN + 4) + KEY_LEN;
    Some(u32::from_le_bytes(body.get(off..off + 4)?.try_into().ok()?))
}
pub fn leaf_key(body: &[u8], i: usize) -> Option<&[u8]> {
    body.get(i * (KEY_LEN + 1)..i * (KEY_LEN + 1) + KEY_LEN)
}
pub fn leaf_live(body: &[u8], i: usize) -> Option<bool> {
    Some(body.get(i * (KEY_LEN + 1) + KEY_LEN)? != &0)
}

pub fn lower(body: &[u8], key: &[u8; KEY_LEN], step: usize, count: usize) -> usize {
    let mut lo = 0;
    let mut hi = count;
    while lo < hi {
        let mid = (lo + hi) / 2;
        if step == KEY_LEN + 4 && dir_leaf(body, mid) == Some(0) {
            hi = mid;
            continue;
        }
        let k = if step == KEY_LEN + 4 {
            dir_key(body, mid)
        } else {
            leaf_key(body, mid)
        };
        if k.is_some_and(|v| v < key) {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    lo
}

pub fn slot(body: &[u8], key: &[u8; KEY_LEN], step: usize, count: usize) -> usize {
    let i = lower(body, key, step, count);
    if i > 0 && dir_key(body, i) != Some(key.as_slice()) {
        i - 1
    } else {
        i
    }
}

pub fn empty_dir() -> Vec<u8> {
    vec![0; DIR_CAP * (KEY_LEN + 4)]
}
