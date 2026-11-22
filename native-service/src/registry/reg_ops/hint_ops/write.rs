use super::{Disk, MutLock, RegErr, Registry, appdata, map_push_err};
impl Registry {
    #[allow(clippy::type_complexity)]
    pub fn hint_ack<D: Disk, L: MutLock>(
        &self,
        disk: &D,
        lock: &L,
        sender_cap: &[u8],
        ns: &[u8],
        key: &[u8],
        seq: u64,
    ) -> Result<(), RegErr<D::Err, (), (), L::Err>> {
        appdata::push::hint_ack(disk, lock, &self.dir, sender_cap, ns, key, seq)
            .map_err(map_push_err)
    }
}
