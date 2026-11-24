//! Load the three-node search path as mutable working entries.
mod util;

use super::{Disk, KEY_LEN, Scope, StoreErr, find, format, mutate, read};

pub struct State {
    pub nd: u32,
    pub nl: u32,
    pub generation: u32,
    pub root: Vec<mutate::DirEntry>,
    pub di: usize,
    pub dir: Vec<mutate::DirEntry>,
    pub leaf: Vec<mutate::LeafEntry>,
}

pub fn load<D: Disk>(
    disk: &D,
    s: Scope,
    key: [u8; KEY_LEN],
    live: bool,
) -> Result<State, StoreErr<D::Err>> {
    let (nd, nl, generation) = util::meta(disk, s)?;
    let raw = if generation == 0 {
        find::empty_dir()
    } else {
        util::node(disk, s.root(generation))?
    };
    let root = mutate::dirs(&raw);
    let di = util::slot(&root, &key);
    let mut dir = if root.is_empty() {
        Vec::new()
    } else {
        mutate::dirs(&util::node(disk, s.dir(root[di].1))?)
    };
    let li = util::slot(&dir, &key);
    let mut leaf = if dir.is_empty() {
        Vec::new()
    } else {
        mutate::leaves(&util::node(disk, s.leaf(dir[li].1))?)
    };
    if !dir.is_empty() {
        dir.remove(li);
    }
    mutate::put_leaf(&mut leaf, key, live);
    Ok(State {
        nd,
        nl,
        generation,
        root,
        di,
        dir,
        leaf,
    })
}
