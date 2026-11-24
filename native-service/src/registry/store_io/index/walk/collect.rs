//! Leaf page collection for one keyset page.
use super::{KEY_LEN, find};

pub fn collect(
    body: &[u8],
    first: bool,
    after: Option<&[u8; KEY_LEN]>,
    out: &mut Vec<[u8; KEY_LEN]>,
    limit: usize,
) {
    for i in 0..body.len() / (KEY_LEN + 1) {
        if out.len() >= limit {
            return;
        }
        let Some(k) = find::leaf_key(body, i) else {
            return;
        };
        if find::leaf_live(body, i) != Some(true) || first && after.is_some_and(|a| k <= a) {
            continue;
        }
        let mut key = [0u8; KEY_LEN];
        key.copy_from_slice(k);
        out.push(key);
    }
}
