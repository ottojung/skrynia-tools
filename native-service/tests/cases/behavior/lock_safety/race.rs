use super::slow_disk::SlowDisk;
use crate::cases::support::{MemDisk, StdLock};
use skrynia_native_service::reg::Registry;
use skrynia_native_service::wire::AppPrecond;
use std::sync::{Arc, Barrier, atomic::AtomicBool, atomic::Ordering};

#[test]
fn slow_disk_mutation_does_not_steal_lock() {
    let gate = Arc::new(Barrier::new(2));
    let armed = Arc::new(AtomicBool::new(true));
    let disk = Arc::new(SlowDisk {
        inner: MemDisk::default(),
        gate: gate.clone(),
        armed: armed.clone(),
    });
    let reg = Arc::new(Registry::open(&*disk, "/r").unwrap());
    let lock = Arc::new(StdLock::default());
    let wc = [7u8; 32];
    let rc = skrynia_native_service::auth::read_cap_vec(&wc);
    let (r1, d1, l1) = (reg.clone(), disk.clone(), lock.clone());
    let t =
        std::thread::spawn(move || r1.app_put(&*d1, &*l1, &wc, b"k", &AppPrecond::Absent, b"a"));
    gate.wait();
    let busy = reg.app_put(&*disk, &*lock, &wc, b"k", &AppPrecond::Absent, b"b");
    assert!(matches!(
        busy,
        Err(skrynia_native_service::reg::RegErr::Busy)
    ));
    gate.wait();
    assert!(t.join().unwrap().is_ok());
    armed.store(false, Ordering::SeqCst);
    let got = reg.app_get(&*disk, &rc, b"k").unwrap();
    assert_eq!(got, b"a".to_vec());
}
