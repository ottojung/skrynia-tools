//! COW commit with retired-node cleanup.
use super::{DIR_CAP, Disk, KEY_LEN, LEAF_CAP, Scope, StoreErr, format, mutate, serialize, state};
pub fn change<D: Disk>(
    disk: &D,
    s: Scope,
    key: [u8; KEY_LEN],
    insert: bool,
) -> Result<(), StoreErr<D::Err>> {
    let Some(mut st) = state::load(disk, s, key, insert)? else {
        return Ok(());
    };
    if st.nl == u32::MAX || st.nd == u32::MAX || st.generation == u32::MAX {
        return Err(StoreErr::BadRecord);
    }
    let mut fresh = Vec::new();
    if !st.leaf.is_empty() {
        if st.leaf.len() > LEAF_CAP {
            let tail = st.leaf.split_off(st.leaf.len() / 2);
            st.nl += 1;
            fresh.push((s.leaf(st.nl), format::seal(serialize::leaf(&tail))));
            serialize::put_dir(&mut st.dir, tail[0].0, st.nl);
        }
        st.nl += 1;
        fresh.push((s.leaf(st.nl), format::seal(serialize::leaf(&st.leaf))));
        serialize::put_dir(&mut st.dir, st.leaf[0].0, st.nl);
    }
    if st.di < st.root.len() {
        st.root.remove(st.di);
    }
    if !st.dir.is_empty() {
        if st.dir.len() > DIR_CAP {
            let tail = st.dir.split_off(st.dir.len() / 2);
            st.nd += 1;
            fresh.push((s.dir(st.nd), format::seal(serialize::dir(&tail))));
            serialize::put_dir(&mut st.root, tail[0].0, st.nd);
        }
        st.nd += 1;
        fresh.push((s.dir(st.nd), format::seal(serialize::dir(&st.dir))));
        serialize::put_dir(&mut st.root, st.dir[0].0, st.nd);
    }
    let generation = st.generation + 1;
    fresh.push((s.root(generation), format::seal(serialize::root(&st.root))));
    for (p, bytes) in fresh {
        disk.write_atomic(&p, &bytes).map_err(StoreErr::Disk)?;
    }
    disk.write_atomic(&s.meta(), &format::make_meta(st.nd, st.nl, generation))
        .map_err(StoreErr::Disk)?;
    state::reclaim::save(disk, s, [st.old_root, st.old_dir, st.old_leaf])?;
    state::reclaim::take(disk, s)
}
