use super::{EventSet, fake::Fake};
#[test]
fn ordered_pagination_does_not_disguise_incomplete_scans() {
    let store = Fake::default();
    let events = EventSet::open(&store, "events").unwrap();
    for i in 1..=8 {
        events.insert(&format!("{i:032x}"), &[i as u8]).unwrap();
    }
    let limited = events.scan(2).unwrap();
    assert_eq!(limited.events.len(), 6);
    assert!(!limited.complete);
    let complete = events.scan(3).unwrap();
    assert!(complete.complete);
    assert_eq!(complete.events.len(), 8);
    assert_eq!(complete.events[0].id, format!("{:032x}", 1));
    assert_eq!(complete.events[7].bytes, vec![8]);
    assert!(events.scan(0).is_err());
}
