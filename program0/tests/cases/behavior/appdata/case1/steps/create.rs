use super::req::{get_req, put_req};
use crate::cases::support::{FsDisk, StdLock};
use skrynia_program0::reg::Registry;
use skrynia_program0::wire::{AppPrecond, STATUS_ERR, STATUS_OK};

pub fn create_and_bad(
    reg: &Registry,
    disk: &FsDisk,
    lock: &StdLock,
    wc: &[u8],
    rc: &[u8],
) -> Vec<u8> {
    let cr = super::super::support::call(reg, disk, lock, put_req(wc, AppPrecond::Absent, b"v1"));
    assert_eq!(cr.status, STATUS_OK);
    let h1 = cr.body.clone();
    let bad_put = put_req(rc, AppPrecond::Present(h1.clone()), b"evil");
    assert_eq!(
        super::super::support::call(reg, disk, lock, bad_put).status,
        STATUS_ERR
    );
    assert_eq!(
        super::super::support::call(reg, disk, lock, get_req(rc)).body,
        b"v1"
    );
    let stale = put_req(wc, AppPrecond::Present(vec![0u8; 32]), b"v2");
    assert_eq!(
        super::super::support::call(reg, disk, lock, stale).status,
        STATUS_ERR
    );
    h1
}
