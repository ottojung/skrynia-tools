use super::count::Counted;
use crate::cases::support::NoLock;
use skrynia_native_service::reg::Registry;

#[test]
fn activation_touches_constant_bytes_even_with_thousand_historical_versions() {
    let d = Counted::default();
    let lock = NoLock;
    let reg = Registry::open(&d, "reg").unwrap();
    let admin = [4u8; 32];
    let iid = reg.register(&d, &lock, &admin, b"first".to_vec()).unwrap();
    let mut hashes = Vec::new();
    for i in 0..1000 {
        let code = format!("version-{i}");
        let hash = reg
            .stage_release(&d, &lock, &admin, code.as_bytes())
            .unwrap();
        reg.seal_release(&d, &lock, &admin, &hash).unwrap();
        hashes.push(hash);
    }
    assert_eq!(reg.list_ready(&d, &iid).unwrap().len(), 1001);
    let before = d.stats();
    let first = reg
        .activate_release(&d, &lock, &admin, 1, &hashes[0])
        .unwrap();
    let after = d.stats();
    assert_eq!(first.revision, 2);
    assert!(after.0 - before.0 <= 100, "must not read any code blob");
    assert_eq!(after.1 - before.1, 0, "must not enumerate releases");
    assert!(after.2 - before.2 <= 42, "write only fixed-size pointer");
    let final_state = reg
        .activate_release(&d, &lock, &admin, 2, &hashes[999])
        .unwrap();
    assert_eq!(final_state.revision, 3);
    assert_eq!(reg.lookup(&d, &iid).unwrap(), b"version-999");
}
