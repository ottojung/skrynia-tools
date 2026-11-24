//! Allocate node ids and build the bounded transaction journal.
use super::{Disk, StoreErr, plan_dir, plan_leaf, state};

pub struct Ids {
    pub nd: u32,
    pub nl: u32,
    pub generation: u32,
    pub leaf: u32,
    pub dir: u32,
    pub leaf_tail: Option<(u32, Vec<u8>)>,
    pub dir_tail: Option<(u32, Vec<u8>)>,
    pub journal: state::reclaim::Journal,
}

pub fn allocate<D: Disk>(st: &mut state::State) -> Result<Ids, StoreErr<D::Err>> {
    let (nl, leaf_tail) = plan_leaf::leaf::<D>(st, st.nl)?;
    let (nd, dir_tail) = plan_dir::dir::<D>(st, st.nd)?;
    let generation = st.generation.checked_add(1).ok_or(StoreErr::BadRecord)?;
    let journal = state::reclaim::Journal {
        target: generation,
        fresh_leaf_head: if st.leaf.is_empty() { 0 } else { nl },
        fresh_leaf_tail: leaf_tail.as_ref().map_or(0, |t| t.0),
        fresh_dir_head: if st.dir.is_empty() { 0 } else { nd },
        fresh_dir_tail: dir_tail.as_ref().map_or(0, |t| t.0),
        old_root: st.old_root,
        old_dir: st.old_dir,
        old_leaf: st.old_leaf,
    };
    Ok(Ids {
        nd,
        nl,
        generation,
        leaf: nl,
        dir: nd,
        leaf_tail,
        dir_tail,
        journal,
    })
}
