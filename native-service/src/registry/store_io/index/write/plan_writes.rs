//! Materialise the planned fresh node writes.
use super::Scope;

pub struct Plan {
    pub generation: u32,
    pub nd: u32,
    pub nl: u32,
    pub leaf: Option<(u32, Vec<u8>)>,
    pub leaf_tail: Option<(u32, Vec<u8>)>,
    pub dir: Option<(u32, Vec<u8>)>,
    pub dir_tail: Option<(u32, Vec<u8>)>,
    pub root: Vec<u8>,
    pub journal: super::state::reclaim::Journal,
}

pub fn writes(s: Scope, p: Plan) -> Vec<(String, Vec<u8>)> {
    let mut out = Vec::new();
    if let Some((id, bytes)) = p.leaf_tail {
        out.push((s.leaf(id), bytes));
    }
    if let Some((id, bytes)) = p.leaf {
        out.push((s.leaf(id), bytes));
    }
    if let Some((id, bytes)) = p.dir_tail {
        out.push((s.dir(id), bytes));
    }
    if let Some((id, bytes)) = p.dir {
        out.push((s.dir(id), bytes));
    }
    out.push((s.root(p.generation), p.root));
    out
}
