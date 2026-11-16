use super::{EventSet, fake::Fake};
#[test]
fn concurrent_distinct_creations_form_grow_only_set() {
    let store = Fake::default();
    let events = EventSet::open(&store, "eventrace").unwrap();
    std::thread::scope(|scope| {
        for n in 0..32u64 {
            let events = &events;
            scope.spawn(move || {
                events
                    .insert(&format!("{n:032x}"), &n.to_be_bytes())
                    .unwrap();
            });
        }
    });
    let scan = events.scan(20).unwrap();
    assert!(scan.complete);
    assert_eq!(scan.events.len(), 32);
    for (n, event) in scan.events.iter().enumerate() {
        assert_eq!(event.id, format!("{n:032x}"));
        assert_eq!(event.bytes, (n as u64).to_be_bytes());
    }
}
