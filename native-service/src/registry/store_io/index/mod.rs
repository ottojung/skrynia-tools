//! Bounded, immutable, page-addressable sorted key index for ordinary services.
//!
//! All reads traverse at most three fixed-width nodes plus a bounded number of
//! leaves, independent of history length. The meta record is the single atomic
//! commit point; every other node is immutable and copy-on-write.
pub(crate) use super::{Disk, StoreErr, read_active};

pub(crate) mod find;
pub(crate) mod format;
pub(crate) mod mutate;
pub(crate) mod page;
pub(crate) mod pending;
pub(crate) mod serialize;
pub(crate) mod state;
pub(crate) mod walk;
pub(crate) mod write;

pub use page::page;
pub use pending::{
    clear as clear_pending, recover, scope, take as take_pending, write as write_pending,
};
pub use state::{ensure_program, preflight};
pub use write::{dry_run, insert, remove};

pub const KEY_LEN: usize = 32;
pub const LEAF_CAP: usize = 124;
pub const DIR_CAP: usize = 128;
pub const ROOT_CAP: usize = 256;

pub(crate) fn read<D: Disk>(disk: &D, path: String) -> Result<Option<Vec<u8>>, StoreErr<D::Err>> {
    disk.read_all(&path).map_err(StoreErr::Disk)
}

#[derive(Clone, Copy)]
pub struct Scope<'a> {
    pub base: &'a str,
}

impl<'a> Scope<'a> {
    fn root_dir(self) -> String {
        format!("{}/index", self.base.trim_end_matches('/'))
    }
    pub fn meta(self) -> String {
        format!("{}/meta", self.root_dir())
    }
    pub fn root(self, generation: u32) -> String {
        format!("{}/r/{}", self.root_dir(), generation)
    }
    pub fn dir(self, id: u32) -> String {
        format!("{}/d/{}", self.root_dir(), id)
    }
    pub fn leaf(self, id: u32) -> String {
        format!("{}/l/{}", self.root_dir(), id)
    }
    pub fn pending(self) -> String {
        format!("{}/pending", self.root_dir())
    }
}
