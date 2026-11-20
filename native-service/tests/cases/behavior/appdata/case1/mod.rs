mod steps;
mod support;
use crate::cases::support::{FsDisk, StdLock};
use skrynia_native_service::auth;
use skrynia_native_service::reg::Registry;
use std::fs;
use support::tmp;

#[test]
fn readonly_token_cannot_put_delete_and_stale_cas_loses_and_persisted() {
    let dir = tmp();
    let disk = FsDisk(dir.clone());
    let lock = StdLock::default();
    let reg = Registry::open(&disk, "p").unwrap();
    let wc = [42u8; 32];
    let rc = auth::read_cap_vec(&wc);
    let reg2 = Registry::open(&disk, "p").unwrap();
    steps::run(&reg, &reg2, &disk, &lock, &wc, &rc);
    let _ = fs::remove_dir_all(dir);
}
