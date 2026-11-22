//! Registry facade for least-privilege sender hint operations.
use super::Disk;
use super::appdata;
use super::map_push_err;
use crate::reg::{RegErr, Registry};

impl Registry {
    #[allow(clippy::type_complexity)]
    pub fn hint_list<D: Disk>(
        &self,
        disk: &D,
        sender_cap: &[u8],
        ns: &[u8],
        key: &[u8],
    ) -> Result<Vec<u8>, RegErr<D::Err, (), (), ()>> {
        appdata::push::hint_list(disk, &self.dir, sender_cap, ns, key).map_err(map_push_err)
    }

    #[allow(clippy::type_complexity)]
    pub fn hint_read<D: Disk>(
        &self,
        disk: &D,
        sender_cap: &[u8],
        ns: &[u8],
        key: &[u8],
        seq: u64,
    ) -> Result<Vec<u8>, RegErr<D::Err, (), (), ()>> {
        appdata::push::hint_read(disk, &self.dir, sender_cap, ns, key, seq).map_err(map_push_err)
    }

    #[allow(clippy::type_complexity)]
    pub fn hint_ack<D: Disk>(
        &self,
        disk: &D,
        sender_cap: &[u8],
        ns: &[u8],
        key: &[u8],
        seq: u64,
    ) -> Result<(), RegErr<D::Err, (), (), ()>> {
        appdata::push::hint_ack(disk, &self.dir, sender_cap, ns, key, seq).map_err(map_push_err)
    }

    /// Read stored subscription material for delivery. Scoped to the exact
    /// (namespace, key) bound to the sender capability; no public/browser path.
    #[allow(clippy::type_complexity)]
    pub fn sub_fetch<D: Disk>(
        &self,
        disk: &D,
        sender_cap: &[u8],
        ns: &[u8],
        key: &[u8],
    ) -> Result<Vec<u8>, RegErr<D::Err, (), (), ()>> {
        appdata::push::sub_fetch(disk, &self.dir, sender_cap, ns, key).map_err(map_push_err)
    }
}
