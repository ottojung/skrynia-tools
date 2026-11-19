use super::MemDisk;
use super::fixture::{call, install};
use skrynia_program0::{auth, cap::Disk, store, wire::STATUS_ERR};

#[test]
fn no_operator_cap_means_no_anonymous_registration_or_disk_growth() {
    let disk = MemDisk::default();
    let admin = [9u8; 32];
    let res = call(&disk, install(&admin));
    assert_eq!(res.status, STATUS_ERR);
    assert_eq!(res.body, [9]);
    let id = auth::read_cap(&admin);
    assert!(
        disk.read_all(&store::active_path("reg", &id).unwrap())
            .unwrap()
            .is_none()
    );
    assert!(disk.read_all("reg/operator.cap").unwrap().is_none());
}

#[test]
fn wrong_operator_cap_cannot_install_but_correct_cap_can() {
    let disk = MemDisk::default();
    disk.write_atomic("reg/operator.cap", &[85u8; 32]).unwrap();
    let admin = [10u8; 32];
    let mut wrong = install(&admin);
    if let skrynia_program0::wire::Request::Register { operator_cap, .. } = &mut wrong {
        operator_cap[0] ^= 1;
    }
    assert_eq!(call(&disk, wrong).body, [9]);
    let good = call(&disk, install(&admin));
    assert_eq!(good.status, 0);
    assert_eq!(good.body, auth::read_cap(&admin));
}
#[test]
fn corrupt_private_operator_cap_fails_closed() {
    let disk = MemDisk::default();
    disk.write_atomic("reg/operator.cap", &[0u8; 31]).unwrap();
    assert_eq!(call(&disk, install(&[8u8; 32])).body, [7]);
}
