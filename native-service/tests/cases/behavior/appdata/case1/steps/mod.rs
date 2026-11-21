mod create;
mod req;
mod update;
use crate::cases::support::{FsDisk, StdLock};
use skrynia_native_service::reg::Registry;

pub fn run(reg: &Registry, reg2: &Registry, disk: &FsDisk, lock: &StdLock, wc: &[u8], rc: &[u8]) {
    let h1 = create::create_and_bad(reg, disk, lock, wc, rc);
    update::update_and_delete(reg, reg2, disk, lock, wc, rc, h1);
}
