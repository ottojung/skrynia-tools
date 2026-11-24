//! Load the three-node search path as mutable working entries.
mod ensure;
mod finish;
mod load;
mod peek;
pub(super) mod reclaim;
mod util;

pub(crate) use super::pending::scope;
use super::{
    DIR_CAP, Disk, KEY_LEN, LEAF_CAP, ROOT_CAP, Scope, StoreErr, find, format, mutate, read,
};
pub use ensure::{ensure_program, preflight};
pub use peek::peek;

pub struct State {
    pub nd: u32,
    pub nl: u32,
    pub generation: u32,
    pub root: Vec<mutate::DirEntry>,
    pub di: usize,
    pub dir: Vec<mutate::DirEntry>,
    pub leaf: Vec<mutate::LeafEntry>,
    pub old_root: u32,
    pub old_dir: u32,
    pub old_leaf: u32,
}

pub use load::load;
