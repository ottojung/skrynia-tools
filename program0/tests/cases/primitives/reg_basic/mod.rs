mod eval;
use crate::cases::support::{MemDisk, NoLock};
use eval::EchoEval;
use skrynia_program0::auth;
use skrynia_program0::reg::Registry;
use skrynia_program0::wire::AppPrecond;

#[test]
fn register_set_invoke_and_recover() {
    let disk = MemDisk::default();
    let lock = NoLock;
    let reg = Registry::open(&disk, "reg").unwrap();
    let admin = [7u8; 32];
    let iid = reg.register(&disk, &lock, &admin, b"v1".to_vec()).unwrap();
    assert_eq!(iid, auth::read_cap_vec(&admin));
    assert_eq!(
        reg.invoke(&disk, &lock, &EchoEval, &iid, b"in").unwrap(),
        b"v1in"
    );
    let h1 = auth::hash_bytes(b"v1").to_vec();
    reg.set(
        &disk,
        &lock,
        &admin,
        &AppPrecond::Present(h1),
        b"v2".to_vec(),
    )
    .unwrap();
    assert_eq!(reg.lookup(&iid).unwrap(), b"v2");
    assert_eq!(
        reg.invoke(&disk, &lock, &EchoEval, &iid, b"x").unwrap(),
        b"v2x"
    );
    let reg2 = Registry::open(&disk, "reg").unwrap();
    assert_eq!(
        reg2.invoke(&disk, &lock, &EchoEval, &iid, b"y").unwrap(),
        b"v2y"
    );
    let h2 = auth::hash_bytes(b"v2").to_vec();
    reg2.revoke(&disk, &lock, &admin, &AppPrecond::Present(h2))
        .unwrap();
    assert!(reg2.lookup(&iid).is_err());
    assert!(
        reg2.set(&disk, &lock, &admin, &AppPrecond::Absent, b"v3".to_vec())
            .is_err()
    );
}
