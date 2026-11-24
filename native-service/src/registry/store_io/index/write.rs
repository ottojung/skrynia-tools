//! Copy-on-write insertion and tombstoning; meta is the single commit point.
use super::{DIR_CAP, Disk, KEY_LEN, Scope, StoreErr, format, mutate, serialize, state};

pub fn insert<D: Disk>(disk: &D, s: Scope, key: [u8; KEY_LEN]) -> Result<(), StoreErr<D::Err>> {
    change(disk, s, key, true)
}

pub fn remove<D: Disk>(disk: &D, s: Scope, key: [u8; KEY_LEN]) -> Result<(), StoreErr<D::Err>> {
    change(disk, s, key, false)
}

fn change<D: Disk>(
    disk: &D,
    s: Scope,
    key: [u8; KEY_LEN],
    live: bool,
) -> Result<(), StoreErr<D::Err>> {
    let mut st = state::load(disk, s, key, live)?;
    let mut fresh = Vec::new();
    if st.leaf.len() > super::LEAF_CAP {
        let tail = st.leaf.split_off(st.leaf.len() / 2);
        st.nl += 1;
        fresh.push((s.leaf(st.nl), format::seal(serialize::leaf(&tail))));
        mutate::put_dir(&mut st.dir, tail[0].0, st.nl);
    }
    st.nl += 1;
    fresh.push((s.leaf(st.nl), format::seal(serialize::leaf(&st.leaf))));
    mutate::put_dir(&mut st.dir, st.leaf[0].0, st.nl);
    if !st.root.is_empty() {
        st.root.remove(st.di);
    }
    if st.dir.len() > DIR_CAP {
        let tail = st.dir.split_off(st.dir.len() / 2);
        st.nd += 1;
        fresh.push((s.dir(st.nd), format::seal(serialize::dir(&tail))));
        mutate::put_dir(&mut st.root, tail[0].0, st.nd);
    }
    st.nd += 1;
    fresh.push((s.dir(st.nd), format::seal(serialize::dir(&st.dir))));
    mutate::put_dir(&mut st.root, st.dir[0].0, st.nd);
    let generation = st.generation + 1;
    fresh.push((s.root(generation), format::seal(serialize::root(&st.root))));
    for (p, bytes) in fresh {
        disk.write_atomic(&p, &bytes).map_err(StoreErr::Disk)?;
    }
    disk.write_atomic(&s.meta(), &format::make_meta(st.nd, st.nl, generation))
        .map_err(StoreErr::Disk)
}
