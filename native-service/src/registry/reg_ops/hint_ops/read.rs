use super::{Disk, MutLock, RegErr, Registry, appdata, map_push_err};
impl Registry {
    #[allow(clippy::type_complexity)]
    pub fn hint_list<D: Disk, L: MutLock>(
        &self,
        disk: &D,
        lock: &L,
        sender_cap: &[u8],
        ns: &[u8],
        key: &[u8],
    ) -> Result<Vec<u8>, RegErr<D::Err, (), (), L::Err>> {
        appdata::push::hint_list(disk, lock, &self.dir, sender_cap, ns, key).map_err(map_push_err)
    }
    #[allow(clippy::type_complexity)]
    pub fn hint_read<D: Disk, L: MutLock>(
        &self,
        disk: &D,
        lock: &L,
        sender_cap: &[u8],
        ns: &[u8],
        key: &[u8],
        seq: u64,
    ) -> Result<Vec<u8>, RegErr<D::Err, (), (), L::Err>> {
        appdata::push::hint_read(disk, lock, &self.dir, sender_cap, ns, key, seq)
            .map_err(map_push_err)
    }
    #[allow(clippy::type_complexity)]
    pub fn sub_fetch<D: Disk, L: MutLock>(
        &self,
        disk: &D,
        lock: &L,
        sender_cap: &[u8],
        ns: &[u8],
        key: &[u8],
    ) -> Result<Vec<u8>, RegErr<D::Err, (), (), L::Err>> {
        appdata::push::sub_fetch(disk, lock, &self.dir, sender_cap, ns, key).map_err(map_push_err)
    }
}
