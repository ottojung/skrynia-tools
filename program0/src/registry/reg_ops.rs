use super::appdata;
use super::cap::{Disk, Eval};
use crate::reg::{RegErr, Registry};
use crate::store_io;

impl Registry {
    pub fn lookup(&self, pid: &[u8]) -> Result<Vec<u8>, RegErr<(), (), ()>> {
        self.code
            .read()
            .unwrap()
            .get(pid)
            .cloned()
            .ok_or(RegErr::Missing)
    }
    pub fn revoke<D: Disk>(&self, disk: &D, pid: &[u8]) -> Result<(), RegErr<D::Err, (), ()>> {
        let mut map = self.code.write().unwrap();
        if map.get(pid).is_none() {
            return Err(RegErr::Missing);
        }
        store_io::del(disk, &self.dir, pid).map_err(RegErr::Store)?;
        map.remove(pid);
        Ok(())
    }
    pub fn invoke<D: Disk, E: Eval>(
        &self,
        disk: &D,
        eval: &E,
        pid: &[u8],
        input: &[u8],
    ) -> Result<Vec<u8>, RegErr<D::Err, E::Err, ()>> {
        let code = self
            .code
            .read()
            .unwrap()
            .get(pid)
            .cloned()
            .ok_or(RegErr::Missing)?;
        let dir = self.dir.clone();
        let app = pid.to_vec();
        #[cfg(target_arch = "wasm32")]
        crate::root_eval::set_cb_app(app.clone());
        let mut cb = move |req: &[u8]| appdata::cb::guest_callback(disk, &dir, &app, req);
        let out = eval.eval(&code, input, Some(&mut cb)).map_err(RegErr::Eval);
        #[cfg(target_arch = "wasm32")]
        crate::root_eval::clear_cb_app();
        out
    }
}
