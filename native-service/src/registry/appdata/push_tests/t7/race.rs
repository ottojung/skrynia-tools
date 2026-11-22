use super::{MemDisk, caps, push};
use std::sync::Arc;

#[test]
fn simultaneous_issuers_cannot_rebind_same_grant() {
    let disk = Arc::new(MemDisk::default());
    let lock = Arc::new(crate::mlock::StdLock::default());
    let (issuer, grant, _) = caps();
    let enemy = [17u8; 32];
    let key = b"apps/demo/notice";
    let subject = [0u8; 32];
    let threads: Vec<_> = [issuer, enemy]
        .into_iter()
        .map(|writer| {
            let disk = disk.clone();
            let lock = lock.clone();
            std::thread::spawn(move || {
                push::issue_grant::<MemDisk, crate::mlock::StdLock>(
                    &disk,
                    &lock,
                    "d",
                    &push::Kind::Sender,
                    &writer,
                    &grant,
                    key,
                    &subject,
                )
            })
        })
        .collect();
    let good = threads
        .into_iter()
        .map(|t| t.join().unwrap().is_ok())
        .filter(|ok| *ok)
        .count();
    assert_eq!(good, 1);
    let binding =
        push::verify_grant::<MemDisk, ()>(&disk, "d", &push::Kind::Sender, &grant, key, &subject)
            .unwrap();
    assert!(binding == crate::read_cap(&issuer) || binding == crate::read_cap(&enemy));
}
