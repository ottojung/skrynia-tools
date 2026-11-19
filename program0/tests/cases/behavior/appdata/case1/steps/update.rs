use super::req::{del_req, get_req, put_req};
use crate::cases::support::{FsDisk, StdLock};
use skrynia_program0::reg::Registry;
use skrynia_program0::wire::{AppPrecond, STATUS_ERR, STATUS_OK};

pub fn update_and_delete(
    reg: &Registry,
    reg2: &Registry,
    disk: &FsDisk,
    lock: &StdLock,
    wc: &[u8],
    rc: &[u8],
    h1: Vec<u8>,
) {
    let ur =
        super::super::support::call(reg, disk, lock, put_req(wc, AppPrecond::Present(h1), b"v2"));
    let h2 = ur.body;
    assert_eq!(
        super::super::support::call(reg, disk, lock, del_req(rc, h2.clone())).status,
        STATUS_ERR
    );
    assert_eq!(
        super::super::support::call(reg2, disk, lock, get_req(rc)).body,
        b"v2"
    );
    assert_eq!(
        super::super::support::call(reg2, disk, lock, del_req(wc, h2)).status,
        STATUS_OK
    );
    assert_eq!(
        super::super::support::call(reg2, disk, lock, get_req(rc)).status,
        STATUS_ERR
    );
}
