//! Bounded, immutable, page-addressable sorted key index for ordinary services.
//!
//! All reads traverse at most three fixed-width nodes plus a bounded number of
//! leaves, independent of history length. The meta record is the single atomic
//! commit point; every other node is immutable and copy-on-write.
use super::{Disk, StoreErr, read_active};

mod find;
mod format;
mod mutate;
mod page;
mod pending;
mod serialize;
mod state;
mod walk;
mod write;

pub use page::page;
pub use pending::{
    clear as clear_pending, recover, scope, take as take_pending, write as write_pending,
};
pub use write::{insert, remove};

pub const KEY_LEN: usize = 32;
pub const LEAF_CAP: usize = 124;
pub const DIR_CAP: usize = 128;
pub const ROOT_CAP: usize = 256;

fn read<D: crate::cap::Disk>(
    disk: &D,
    path: String,
) -> Result<Option<Vec<u8>>, crate::store_io::StoreErr<D::Err>> {
    disk.read_all(&path)
        .map_err(crate::store_io::StoreErr::Disk)
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
