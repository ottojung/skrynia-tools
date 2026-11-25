//! Bounded transaction journal codec and best-effort node reclamation.
pub const MAGIC: &[u8; 4] = b"IXJ1";

pub struct Journal {
    pub target: u32,
    pub fresh_leaf_head: u32,
    pub fresh_leaf_tail: u32,
    pub fresh_dir_head: u32,
    pub fresh_dir_tail: u32,
    pub old_root: u32,
    pub old_dir: u32,
    pub old_leaf: u32,
}

pub fn path(s: super::super::Scope) -> String {
    format!("{}/index/journal", s.base.trim_end_matches('/'))
}

pub fn write<D: super::super::Disk>(
    disk: &D,
    s: super::super::Scope,
    j: &Journal,
) -> Result<(), super::super::StoreErr<D::Err>> {
    let mut b = Vec::with_capacity(40);
    b.extend_from_slice(MAGIC);
    for n in [
        j.target,
        j.fresh_leaf_head,
        j.fresh_leaf_tail,
        j.fresh_dir_head,
        j.fresh_dir_tail,
        j.old_root,
        j.old_dir,
        j.old_leaf,
        0,
    ] {
        b.extend_from_slice(&n.to_le_bytes());
    }
    disk.write_atomic(&path(s), &super::super::format::seal(b))
        .map_err(super::super::StoreErr::Disk)
}
