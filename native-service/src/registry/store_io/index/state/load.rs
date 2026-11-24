//! Load nodes along one search path.
use super::{
    DIR_CAP, Disk, KEY_LEN, LEAF_CAP, ROOT_CAP, Scope, StoreErr, find, mutate, reclaim, util,
};

pub fn load<D: Disk>(
    disk: &D,
    s: Scope,
    key: [u8; KEY_LEN],
    insert: bool,
) -> Result<Option<super::State>, StoreErr<D::Err>> {
    reclaim::recover(disk, s)?;
    let (nd, nl, generation) = util::meta(disk, s)?;
    let raw = if generation == 0 {
        find::empty_dir()
    } else {
        util::node(disk, s.root(generation), ROOT_CAP, KEY_LEN + 4)?
    };
    let root = mutate::dirs(&raw);
    let di = util::slot(&root, &key);
    let old_dir = root.get(di).map(|e| e.1).unwrap_or(0);
    let mut dir = if di >= root.len() {
        Vec::new()
    } else {
        mutate::dirs(&util::node(disk, s.dir(old_dir), DIR_CAP, KEY_LEN + 4)?)
    };
    let li = util::slot(&dir, &key);
    let old_leaf = dir.get(li).map(|e| e.1).unwrap_or(0);
    let mut leaf = if li >= dir.len() {
        Vec::new()
    } else {
        mutate::leaves(&util::node(disk, s.leaf(old_leaf), LEAF_CAP, KEY_LEN + 1)?)
    };
    super::finish::build::<D>(super::finish::Parts {
        key,
        nd,
        nl,
        generation,
        root,
        di,
        dir,
        leaf,
        old_dir,
        old_leaf,
        insert,
        li,
    })
}
