//! Adversarial cross-key wake reproduction on current main (issue 5z confine).
mod common;
use common::crosskey as c;
use common::crosskey_query as q;

#[test]
fn delegated_cross_ns_wake() {
    let d = common::Mem::default();
    c::seed(&d);
    for key in [c::A, c::B, b"thread-0113-vitalik-v1" as &[u8]] {
        assert!(c::put(&d, key, &common::caps().0));
    }
    assert_eq!(q::gens(&d), vec![3]);
    assert_eq!(common::hints(&d, common::WAKE), 0);
    assert!(!q::slot(&d, &q::sns(), c::B));
}

#[test]
fn unauthorised_source_cannot_wake() {
    let d = common::Mem::default();
    c::seed(&d);
    assert!(!c::put(&d, c::B, &[9u8; 32]));
    assert_eq!(q::gens(&d), Vec::<u64>::new());
    assert!(c::put(&d, c::A, &common::caps().0));
    assert_eq!(q::gens(&d), vec![1]);
}

#[test]
fn exact_key_only() {
    let d = common::Mem::default();
    c::seed(&d);
    q::sub_exact(&d, c::A);
    assert!(c::put(&d, c::B, &common::caps().0));
    assert_eq!(common::hints(&d, c::A), 0);
    assert_eq!(q::gens(&d), vec![1]);
}

#[test]
fn bounded_generation() {
    let d = common::Mem::default();
    c::seed(&d);
    for i in 0..1100 {
        assert!(c::put(
            &d,
            format!("thread-{i:04}-v1").as_bytes(),
            &common::caps().0
        ));
    }
    assert_eq!(q::gens(&d), vec![1100]);
    assert_eq!(common::hints(&d, common::WAKE), 0);
}
