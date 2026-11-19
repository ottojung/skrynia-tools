use super::appdata;
use super::{Disk, Eval, MutLock, RegErr, Registry};

impl Registry {
    pub fn lookup(&self, invoke_id: &[u8]) -> Result<Vec<u8>, RegErr<(), (), (), ()>> {
        self.code
            .read()
            .unwrap()
            .get(invoke_id)
            .cloned()
            .ok_or(RegErr::Missing)
    }

    pub fn invoke<D: Disk, L: MutLock, E: Eval>(
        &self,
        disk: &D,
        lock: &L,
        eval: &E,
        invoke_id: &[u8],
        input: &[u8],
    ) -> Result<Vec<u8>, RegErr<D::Err, E::Err, (), L::Err>> {
        let code = self
            .code
            .read()
            .unwrap()
            .get(invoke_id)
            .cloned()
            .ok_or(RegErr::Missing)?;
        let dir = self.dir.clone();
        let iid = invoke_id.to_vec();
        #[cfg(target_arch = "wasm32")]
        crate::root_eval::set_cb_invoke(iid.clone());
        let mut cb = move |req: &[u8]| appdata::cb::guest_callback(disk, lock, &dir, &iid, req);
        let out = eval.eval(&code, input, Some(&mut cb)).map_err(RegErr::Eval);
        #[cfg(target_arch = "wasm32")]
        crate::root_eval::clear_cb_invoke();
        out
    }
}
