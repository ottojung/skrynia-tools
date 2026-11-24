//! Journal-before-write COW commit; committed meta selects reclamation.
use super::{Disk, KEY_LEN, Scope, StoreErr, format, plan, plan_nodes, plan_writes, state};

pub fn change<D: Disk>(
    disk: &D,
    s: Scope,
    key: [u8; KEY_LEN],
    insert: bool,
) -> Result<(), StoreErr<D::Err>> {
    let Some(mut st) = state::load(disk, s, key, insert)? else {
        return Ok(());
    };
    let ids = plan::allocate::<D>(&mut st)?;
    let p = plan_nodes::build(&st, ids);
    let (nd, nl, generation) = (p.nd, p.nl, p.generation);
    state::reclaim::write(disk, s, &p.journal)?;
    for (path, bytes) in plan_writes::writes(s, p) {
        disk.write_atomic(&path, &bytes).map_err(StoreErr::Disk)?;
    }
    disk.write_atomic(&s.meta(), &format::make_meta(nd, nl, generation))
        .map_err(StoreErr::Disk)?;
    state::reclaim::recover(disk, s)
}
