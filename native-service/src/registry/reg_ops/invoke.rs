use super::appdata;
use super::{Disk, Eval, MutLock, RegErr, Registry};
use crate::store_io::{self, StoreErr};

impl Registry {
    pub fn lookup<D: Disk>(
        &self,
        disk: &D,
        invoke_id: &[u8],
    ) -> Result<Vec<u8>, RegErr<D::Err, (), (), ()>> {
        store_io::read_code(disk, &self.dir, invoke_id).map_err(|e| match e {
            StoreErr::Disk(d) => RegErr::Store(StoreErr::Disk(d)),
            _ => RegErr::Missing,
        })
    }

    #[allow(clippy::type_complexity)]
    pub fn invoke<D: Disk, L: MutLock, E: Eval>(
        &self,
        disk: &D,
        lock: &L,
        eval: &E,
        invoke_id: &[u8],
        input: &[u8],
    ) -> Result<Vec<u8>, RegErr<D::Err, E::Err, (), L::Err>> {
        let code = store_io::read_code(disk, &self.dir, invoke_id).map_err(|e| match e {
            StoreErr::Disk(d) => RegErr::Store(StoreErr::Disk(d)),
            _ => RegErr::Missing,
        })?;
        let dir = self.dir.clone();
        let iid = invoke_id.to_vec();
        #[cfg(all(target_arch = "wasm32", not(feature = "cb-guest")))]
        crate::root_eval::set_cb_invoke(iid.clone());
        let mut cb = move |req: &[u8]| appdata::cb::guest_callback(disk, lock, &dir, &iid, req);
        let out = eval.eval(&code, input, Some(&mut cb)).map_err(RegErr::Eval);
        #[cfg(all(target_arch = "wasm32", not(feature = "cb-guest")))]
        crate::root_eval::clear_cb_invoke();
        out
    }
}
