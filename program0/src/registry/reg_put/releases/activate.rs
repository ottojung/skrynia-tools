use super::*;
use crate::store::ActiveState;

impl Registry {
    #[allow(clippy::type_complexity)]
    pub fn activate_release<D: Disk, L: MutLock>(
        &self,
        disk: &D,
        lock: &L,
        admin: &[u8],
        expected_revision: u64,
        target: &[u8; 32],
    ) -> Result<ActiveState, RegErr<D::Err, (), (), L::Err>> {
        if admin.len() != 32 || expected_revision == 0 {
            return Err(RegErr::BadAuth);
        }
        let iid = store::invoke_id(admin).to_vec();
        with_prog_lock(lock, || {
            let Some(current) =
                store_io::read_record(disk, &self.dir, &iid).map_err(RegErr::Store)?
            else {
                return Err(RegErr::Missing);
            };
            if !current.live {
                return Err(RegErr::Missing);
            }
            if current.revision != expected_revision {
                return Err(RegErr::Conflict);
            }
            if !store_io::is_ready(disk, &self.dir, &iid, target).map_err(RegErr::Store)? {
                return Err(RegErr::Missing);
            }
            let revision = current.revision.checked_add(1).ok_or(RegErr::Conflict)?;
            let next = ActiveState {
                hash: *target,
                revision,
                live: true,
            };
            store_io::write_record(disk, &self.dir, &iid, &next).map_err(RegErr::Store)?;
            Ok(next)
        })
    }
}
