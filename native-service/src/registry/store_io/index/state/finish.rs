//! Apply the key change and build the working state.
use super::{Disk, KEY_LEN, State, StoreErr, mutate};

pub struct Parts {
    pub key: [u8; KEY_LEN],
    pub nd: u32,
    pub nl: u32,
    pub generation: u32,
    pub root: Vec<mutate::DirEntry>,
    pub di: usize,
    pub dir: Vec<mutate::DirEntry>,
    pub leaf: Vec<mutate::LeafEntry>,
    pub old_dir: u32,
    pub old_leaf: u32,
    pub insert: bool,
    pub li: usize,
}

pub fn build<D: Disk>(mut p: Parts) -> Result<Option<State>, StoreErr<D::Err>> {
    let found = if p.insert {
        mutate::put_leaf(&mut p.leaf, p.key, true);
        true
    } else {
        mutate::remove_leaf(&mut p.leaf, &p.key)
    };
    if !p.insert && !found {
        return Ok(None);
    }
    if p.li < p.dir.len() {
        p.dir.remove(p.li);
    }
    Ok(Some(State {
        nd: p.nd,
        nl: p.nl,
        generation: p.generation,
        root: p.root,
        di: p.di,
        dir: p.dir,
        leaf: p.leaf,
        old_root: p.generation,
        old_dir: p.old_dir,
        old_leaf: p.old_leaf,
    }))
}
