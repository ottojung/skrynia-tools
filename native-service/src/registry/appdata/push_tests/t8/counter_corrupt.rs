use super::Disk;
use super::{AppPrecond, MemDisk, State, WriteAuth, caps, io, ops, push, seed_sender};

#[test]
fn invalid_highwater_must_fail_closed_before_cas() {
    let disk = MemDisk::default();
    let (writer, _, _) = caps();
    let sender = [21u8; 32];
    let key = b"apps/demo/notice";
    let ns = crate::read_cap(&writer);
    seed_sender(&disk, &sender, key);
    let path = push::hint_counter_path("d", &ns, key);
    disk.write_atomic(&path, b"bad-state").unwrap();
    let got = ops::put(
        &disk,
        &crate::NoLock,
        "d",
        WriteAuth::External(&writer),
        key,
        &AppPrecond::Absent,
        b"value",
    );
    assert!(got.is_err(), "bad durable counter must prevent CAS");
    assert_eq!(io::read_state(&disk, "d", &ns, key).unwrap(), State::Absent);
}
