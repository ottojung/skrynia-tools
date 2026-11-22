use super::{MemDisk, caps, push, seed_sender};
use crate::MutLock;

#[test]
fn sender_cannot_read_staged_hint_during_active_cas() {
    let disk = MemDisk::default();
    let (writer, _, _) = caps();
    let lock = crate::mlock::StdLock::default();
    let sender = [21u8; 32];
    let key = b"apps/demo/notice";
    let ns = crate::read_cap(&writer);
    seed_sender(&disk, &sender, key);
    assert!(lock.try_acquire().unwrap());
    assert!(push::hint_list(&disk, &lock, "d", &sender, &ns, key).is_err());
    assert!(lock.release().is_ok());
    assert!(push::hint_list(&disk, &lock, "d", &sender, &ns, key).is_ok());
}
