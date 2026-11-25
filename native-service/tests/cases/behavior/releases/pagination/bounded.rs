//! A full page reads bounded nodes and never enumerates the directory.
use super::list_all;
use super::support::Calls;
use crate::cases::support::NoLock;
use skrynia_native_service::reg::Registry;
use std::sync::atomic::Ordering;

#[test]
fn page_reads_are_bounded_and_never_enumerate_directory() {
    let d = Calls::default();
    let reg = Registry::open(&d, "reg").unwrap();
    let lock = NoLock;
    let admin = [7u8; 32];
    let iid = reg.register(&d, &lock, &admin, b"first".to_vec()).unwrap();
    for i in 0..1000 {
        let code = format!("v-{i}");
        let h = reg
            .stage_release(&d, &lock, &admin, code.as_bytes())
            .unwrap();
        reg.seal_release(&d, &lock, &admin, &h).unwrap();
    }
    let before = d.calls.load(Ordering::SeqCst);
    let page = reg.list_ready(&d, &iid, None, 256).unwrap();
    let used = d.calls.load(Ordering::SeqCst) - before;
    assert_eq!(page.len(), 256);
    assert!(used <= 32, "bounded page used {used} disk calls");
    assert_eq!(list_all(&reg, &d, &iid).len(), 1001);
}
