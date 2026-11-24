//! Wake namespace writer issues narrow wake-only grant for a source namespace.
use super::{Disk, appdata, map_push_err};
use crate::MutLock;
use crate::reg::{RegErr, Registry};
impl Registry {
    #[allow(clippy::type_complexity)]
    pub fn push_wake_grant<D: Disk, L: MutLock>(
        &self,
        disk: &D,
        lock: &L,
        write_cap: &[u8],
        grant_cap: &[u8],
        key: &[u8],
        source_ns: &[u8],
    ) -> Result<(), RegErr<D::Err, (), (), L::Err>> {
        if source_ns.len() != crate::CAP_LEN {
            return Err(RegErr::BadAuth);
        }
        appdata::push::issue_grant(
            disk,
            lock,
            &self.dir,
            &appdata::push::Kind::Wake,
            write_cap,
            grant_cap,
            key,
            source_ns,
        )
        .map_err(map_push_err)
    }
}
