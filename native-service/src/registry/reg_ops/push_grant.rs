//! Only a namespace writer may issue narrow subscribe/sender grants.
use super::Disk;
use super::appdata;
use super::map_push_err;
use crate::reg::{RegErr, Registry};

impl Registry {
    #[allow(clippy::type_complexity)]
    pub fn push_grant<D: Disk>(
        &self,
        disk: &D,
        write_cap: &[u8],
        grant_cap: &[u8],
        key: &[u8],
        subscriber: &[u8],
    ) -> Result<(), RegErr<D::Err, (), (), ()>> {
        if subscriber.is_empty() || subscriber.len() > 256 {
            return Err(RegErr::BadAuth);
        }
        appdata::push::issue_grant(
            disk,
            &self.dir,
            &appdata::push::Kind::Subscribe,
            write_cap,
            grant_cap,
            key,
            &crate::hash_bytes(subscriber),
        )
        .map_err(map_push_err)
    }

    #[allow(clippy::type_complexity)]
    pub fn push_sender_grant<D: Disk>(
        &self,
        disk: &D,
        write_cap: &[u8],
        sender_cap: &[u8],
        key: &[u8],
    ) -> Result<(), RegErr<D::Err, (), (), ()>> {
        appdata::push::issue_grant(
            disk,
            &self.dir,
            &appdata::push::Kind::Sender,
            write_cap,
            sender_cap,
            key,
            &[0u8; 32],
        )
        .map_err(map_push_err)
    }
}
