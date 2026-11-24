//! Sender grant issuance is a separate ordinary-userland operation.
use super::Disk;
use super::appdata;
use super::map_push_err;
use crate::MutLock;
use crate::reg::{RegErr, Registry};

impl Registry {
    #[allow(clippy::type_complexity)]
    pub fn push_sender_grant<D: Disk, L: MutLock>(
        &self,
        disk: &D,
        lock: &L,
        write_cap: &[u8],
        sender_cap: &[u8],
        key: &[u8],
    ) -> Result<(), RegErr<D::Err, (), (), L::Err>> {
        appdata::push::issue_grant(
            disk,
            lock,
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
