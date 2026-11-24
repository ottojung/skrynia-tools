//! Bounded root-to-leaf keyset traversal for a single page.
mod collect;

use super::{DIR_CAP, Disk, KEY_LEN, ROOT_CAP, Scope, StoreErr, find, read};

pub fn walk<D: Disk>(
    disk: &D,
    s: Scope,
    root: &[u8],
    key: [u8; KEY_LEN],
    after: Option<&[u8; KEY_LEN]>,
    limit: u16,
) -> Result<Vec<[u8; KEY_LEN]>, StoreErr<D::Err>> {
    let start_dir = find::slot(root, &key, KEY_LEN + 4, ROOT_CAP);
    let mut out = Vec::new();
    for d in start_dir..ROOT_CAP {
        if out.len() >= limit as usize {
            break;
        }
        let Some(id) = find::dir_leaf(root, d).filter(|v| *v != 0) else {
            continue;
        };
        let raw = read(disk, s.dir(id))?.ok_or(StoreErr::Missing)?;
        let dir = super::format::open(&raw).ok_or(StoreErr::BadRecord)?;
        let first_dir = d == start_dir;
        let start_leaf = if first_dir {
            find::slot(&dir, &key, KEY_LEN + 4, DIR_CAP)
        } else {
            0
        };
        for l in start_leaf..DIR_CAP {
            if out.len() >= limit as usize {
                break;
            }
            let Some(lid) = find::dir_leaf(&dir, l).filter(|v| *v != 0) else {
                continue;
            };
            let raw = read(disk, s.leaf(lid))?.ok_or(StoreErr::Missing)?;
            let leaf = super::format::open(&raw).ok_or(StoreErr::BadRecord)?;
            collect::collect(
                &leaf,
                first_dir && l == start_leaf,
                after,
                &mut out,
                limit as usize,
            );
        }
    }
    Ok(out)
}
