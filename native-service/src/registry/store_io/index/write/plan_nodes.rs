//! Assemble encoded nodes into the planned write set.
use super::{format, plan, plan_writes, serialize, state};

pub fn build(st: &state::State, ids: plan::Ids) -> plan_writes::Plan {
    let leaf = (!st.leaf.is_empty()).then(|| (ids.leaf, format::seal(serialize::leaf(&st.leaf))));
    let dir = (!st.dir.is_empty()).then(|| (ids.dir, format::seal(serialize::dir(&st.dir))));
    plan_writes::Plan {
        generation: ids.generation,
        nd: ids.nd,
        nl: ids.nl,
        leaf,
        leaf_tail: ids.leaf_tail,
        dir,
        dir_tail: ids.dir_tail,
        root: format::seal(serialize::root(&st.root)),
        journal: ids.journal,
    }
}
