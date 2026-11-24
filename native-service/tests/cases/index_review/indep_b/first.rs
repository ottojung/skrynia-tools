//! First registration is replay-safe at every durable write boundary.
use super::first_support::CrashDisk;
use skrynia_native_service::{mlock::StdLock, reg::Registry};

#[test]
fn first_register_recovers_at_every_durable_boundary() {
    use sha2::{Digest, Sha256};
    let cap = [9; 32];
    let id: [u8; 32] = Sha256::digest(cap).into();
    for after in [false, true] {
        for at in 1..=32 {
            let d = CrashDisk::default();
            let r = Registry::open(&d, "reg").unwrap();
            let l = StdLock::default();
            d.arm(at, after);
            let first = r.register(&d, &l, &cap, b"code".to_vec());
            d.recover_io();
            let exists = r.active_info(&d, &id).is_ok();
            let list = r.list_programs(&d, None, 64).unwrap_or_else(|_| {
                panic!(
                    "list recovery failed at={at} after={after} exists={exists} register_ok={}",
                    first.is_ok()
                )
            });
            assert_eq!(
                list,
                if exists { vec![id] } else { vec![] },
                "at={at} after={after}"
            );
            if !exists {
                r.register(&d, &l, &cap, b"code".to_vec())
                    .unwrap_or_else(|_| panic!("retry failed at={at} after={after}"));
            }
            assert_eq!(r.list_programs(&d, None, 64).unwrap(), vec![id]);
            assert_eq!(r.list_ready(&d, &id, None, 64).unwrap().len(), 1);
        }
    }
}
